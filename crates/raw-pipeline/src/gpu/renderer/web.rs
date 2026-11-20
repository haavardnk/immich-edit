use parking_lot::MutexGuard;

use super::GpuRenderer;
use super::display::DisplayFrame;
use super::masks::MaskAtlas;
use super::meta::MetaRequest;
use crate::PipelineResult;
use crate::gpu::resources::OutputTargets;
use crate::gpu::timer::RenderTimings;
use crate::histogram::Histogram;
use crate::scopes::ScopeGrids;
use crate::timing::StageTiming;

pub struct DisplayMeta {
    pub dims: (u32, u32),
    pub source_dims: (u32, u32),
    pub histogram: Option<Histogram>,
    pub linear_histogram: Option<Histogram>,
    pub scopes: Option<ScopeGrids>,
    pub timings: Vec<StageTiming>,
}

pub struct SubmittedDisplay<'a> {
    renderer: &'a GpuRenderer,
    targets: MutexGuard<'a, Vec<OutputTargets>>,
    bins: MetaRequest,
    dims: (u32, u32),
    source_dims: (u32, u32),
    atlases: [Option<MaskAtlas>; 2],
    timings: RenderTimings<'a>,
}

impl GpuRenderer {
    pub fn submit_display<'a>(&'a self, frame: DisplayFrame<'a>) -> SubmittedDisplay<'a> {
        let DisplayFrame {
            encoder,
            targets,
            slot,
            bins,
            dims,
            source_dims,
            atlases,
            timings,
            sharpen,
            scratch,
            retained,
        } = frame;
        self.ctx.queue.submit(Some(encoder.finish()));
        drop((slot, sharpen, scratch, retained));
        SubmittedDisplay {
            renderer: self,
            targets,
            bins,
            dims,
            source_dims,
            atlases,
            timings,
        }
    }
}

impl SubmittedDisplay<'_> {
    #[expect(
        clippy::await_holding_lock,
        reason = "the frame owns its pooled targets until the bins are read, and the web pool refuses a second frame"
    )]
    pub async fn read_meta(self) -> PipelineResult<DisplayMeta> {
        let Self {
            renderer,
            targets,
            bins,
            dims,
            source_dims,
            atlases,
            timings,
        } = self;
        let counts = renderer.read_meta_counts_async(&targets[0], bins).await?;
        drop(targets);
        let [preview_atlas, layer_atlas] = atlases;
        renderer.release_mask_atlas(preview_atlas);
        renderer.release_mask_atlas(layer_atlas);
        let (histogram, linear_histogram) = counts.histograms();
        let scopes = counts.scopes(timings.clock());
        Ok(DisplayMeta {
            dims,
            source_dims,
            histogram,
            linear_histogram,
            scopes,
            timings: timings.finish_async().await,
        })
    }
}
