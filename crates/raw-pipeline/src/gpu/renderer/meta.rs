use std::ops::Range;

use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BufferUsages, CommandEncoder, Texture, TextureFormat, TextureViewDescriptor};

use super::GpuRenderer;
use crate::PipelineResult;
use crate::gpu::dispatch::{bind_group, buf, dispatch_2d, tex};
use crate::gpu::passes::meta_bins::{BinParams, HISTOGRAM_TILE, MetaBinsPasses, SCOPES_GROUP};
#[cfg(feature = "native")]
use crate::gpu::readback::read_u32_ranges;
#[cfg(feature = "web")]
use crate::gpu::readback::read_u32_ranges_async;
use crate::gpu::resources::{HISTOGRAM_BYTES, OutputTargets, SCOPE_BYTES};
use crate::gpu::timer::RenderTimings;
use crate::histogram::{BINS, Histogram, sample_step};
use crate::scopes::{ScopeGrids, row_step};
use crate::timing::{self, StageClock};

#[derive(Clone, Copy)]
pub(super) struct MetaRequest {
    pub histogram: bool,
    pub scopes: bool,
}

#[derive(Default)]
pub(super) struct MetaCounts {
    histogram: Option<Vec<u32>>,
    scopes: Option<Vec<u32>>,
}

impl MetaCounts {
    pub(super) fn histograms(&self) -> (Option<Histogram>, Option<Histogram>) {
        match &self.histogram {
            Some(counts) => {
                let (display, linear) = counts.split_at(4 * BINS);
                (
                    Some(Histogram::from_counts(display)),
                    Some(Histogram::from_counts(linear)),
                )
            }
            None => (None, None),
        }
    }

    pub(super) fn scopes(&self, clock: &StageClock) -> Option<ScopeGrids> {
        self.scopes
            .as_ref()
            .map(|counts| clock.time(timing::SCOPES, || ScopeGrids::from_counts(counts)))
    }
}

impl GpuRenderer {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn encode_meta_bins(
        &self,
        encoder: &mut CommandEncoder,
        p: &OutputTargets,
        display_src: &Texture,
        linear_src: &Texture,
        request: MetaRequest,
        out_dims: (u32, u32),
        t: &RenderTimings,
    ) {
        let passes = match display_src.format() {
            TextureFormat::Rgba16Uint => &self.passes.depth16(&self.ctx).meta_bins,
            _ => &self.passes.meta_bins,
        };
        let display_view = display_src.create_view(&TextureViewDescriptor::default());
        if request.histogram {
            t.stage(timing::HISTOGRAM, || {
                self.encode_histogram(encoder, p, passes, &display_view, linear_src, out_dims)
            });
        }
        if request.scopes {
            t.stage(timing::SCOPES, || {
                self.encode_scopes(encoder, p, passes, &display_view, out_dims)
            });
        }
    }

    fn encode_histogram(
        &self,
        encoder: &mut CommandEncoder,
        p: &OutputTargets,
        passes: &MetaBinsPasses,
        display_view: &wgpu::TextureView,
        linear_src: &Texture,
        (out_w, out_h): (u32, u32),
    ) {
        let step = sample_step(out_w as usize * out_h as usize) as u32;
        let params = self.bin_params(out_w, out_h, step);
        let linear_view = linear_src.create_view(&TextureViewDescriptor::default());
        let bind = bind_group(
            &self.ctx.device,
            "histogram-bg",
            &passes.histogram_layout,
            &[
                params.as_entire_binding(),
                tex(display_view),
                tex(&linear_view),
                buf(&p.histogram_counts),
            ],
        );
        encoder.clear_buffer(&p.histogram_counts, 0, None);
        dispatch_2d(
            encoder,
            "histogram",
            &passes.histogram,
            &bind,
            out_w.div_ceil(HISTOGRAM_TILE),
            out_h.div_ceil(HISTOGRAM_TILE),
        );
        encoder.copy_buffer_to_buffer(&p.histogram_counts, 0, &p.meta_readback, 0, HISTOGRAM_BYTES);
    }

    fn encode_scopes(
        &self,
        encoder: &mut CommandEncoder,
        p: &OutputTargets,
        passes: &MetaBinsPasses,
        display_view: &wgpu::TextureView,
        (out_w, out_h): (u32, u32),
    ) {
        let step = row_step(out_w as usize, out_h as usize) as u32;
        let params = self.bin_params(out_w, out_h, step);
        let bind = bind_group(
            &self.ctx.device,
            "scopes-bg",
            &passes.scopes_layout,
            &[
                params.as_entire_binding(),
                tex(display_view),
                buf(&p.scope_counts),
            ],
        );
        encoder.clear_buffer(&p.scope_counts, 0, None);
        dispatch_2d(
            encoder,
            "scopes",
            &passes.scopes,
            &bind,
            out_w.div_ceil(SCOPES_GROUP),
            out_h.div_ceil(SCOPES_GROUP),
        );
        encoder.copy_buffer_to_buffer(
            &p.scope_counts,
            0,
            &p.meta_readback,
            HISTOGRAM_BYTES,
            SCOPE_BYTES,
        );
    }

    fn bin_params(&self, out_w: u32, out_h: u32, step: u32) -> wgpu::Buffer {
        let params = BinParams {
            size: [out_w, out_h],
            step,
            _pad: 0,
        };
        self.ctx.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("meta-bins-uniform"),
            contents: bytemuck::bytes_of(&params),
            usage: BufferUsages::UNIFORM,
        })
    }

    #[cfg(feature = "native")]
    pub(super) fn read_meta_counts(
        &self,
        p: &OutputTargets,
        request: MetaRequest,
        cancel: Option<&crate::cancel::CancelToken>,
    ) -> PipelineResult<MetaCounts> {
        let ranges = request.ranges();
        if ranges.is_empty() {
            return Ok(MetaCounts::default());
        }
        let counts = read_u32_ranges(&self.ctx, &p.meta_readback, &ranges, cancel)?;
        Ok(request.collect(counts))
    }

    #[cfg(feature = "web")]
    pub(super) async fn read_meta_counts_async(
        &self,
        p: &OutputTargets,
        request: MetaRequest,
    ) -> PipelineResult<MetaCounts> {
        let ranges = request.ranges();
        if ranges.is_empty() {
            return Ok(MetaCounts::default());
        }
        let counts = read_u32_ranges_async(&p.meta_readback, &ranges).await?;
        Ok(request.collect(counts))
    }
}

impl MetaRequest {
    fn ranges(self) -> Vec<Range<u64>> {
        [
            self.histogram.then_some(0..HISTOGRAM_BYTES),
            self.scopes
                .then_some(HISTOGRAM_BYTES..HISTOGRAM_BYTES + SCOPE_BYTES),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    fn collect(self, counts: Vec<Vec<u32>>) -> MetaCounts {
        let mut counts = counts.into_iter();
        let histogram = self.histogram.then(|| counts.next()).flatten();
        let scopes = self.scopes.then(|| counts.next()).flatten();
        MetaCounts { histogram, scopes }
    }
}

#[cfg(test)]
mod tests;
