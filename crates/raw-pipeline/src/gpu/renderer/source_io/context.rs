use std::sync::Arc;

use half::f16;
use wgpu::{Texture, TextureFormat, TextureUsages};

use super::texel_size;
#[cfg(feature = "native")]
use crate::cancel::CancelToken;
use crate::gpu::limits::check_texture;
use crate::gpu::passes::dehaze::MOMENT_FORMAT;
use crate::gpu::passes::presence::CLARITY_GUIDE_FORMAT;
use crate::gpu::renderer::GpuRenderer;
use crate::gpu::renderer::display::{GlobalContext, GridView, HazeGrid, LevelView};
use crate::gpu::texture::{full_view, texture_2d, write_texture_2d};
use crate::source::{ContextGrid, ContextPart, SourceHeader};
#[cfg(feature = "native")]
use crate::source::{ContextPlane, ContextPlanes};
#[cfg(feature = "native")]
use crate::tile::Rect;
use crate::{PipelineError, PipelineResult};

impl GpuRenderer {
    pub(super) fn upload_context(
        &self,
        header: &SourceHeader,
        samples: &[f32],
    ) -> PipelineResult<Option<Arc<GlobalContext>>> {
        let Some(shape) = header.context else {
            return Ok(None);
        };
        if samples.len() as u64 != shape.samples() {
            return Err(PipelineError::Render(format!(
                "source context holds {} samples, expected {}",
                samples.len(),
                shape.samples()
            )));
        }
        let mut context = GlobalContext::default();
        let mut rest = samples;
        for (part, grid) in shape.grids() {
            let texels = grid.dims.0 as usize * grid.dims.1 as usize;
            let (data, tail) = rest.split_at(texels * part.channels());
            rest = tail;
            let texture = self.upload_grid(part, grid, data)?;
            let view = GridView {
                view: full_view(&texture),
                origin: grid.origin,
            };
            let level = LevelView {
                grid: view.clone(),
                level: grid.level,
            };
            match part {
                ContextPart::Haze => {
                    let atmosphere = header.atmosphere.ok_or_else(|| {
                        PipelineError::Render("a source haze grid needs an atmosphere".into())
                    })?;
                    context.haze = Some(HazeGrid {
                        atmosphere,
                        ab: view,
                    });
                }
                ContextPart::Texture => context.presence.texture = Some(level),
                ContextPart::Guide => context.presence.guide = Some(level),
                ContextPart::Shadows => context.shadows = Some(level),
            }
        }
        Ok(Some(Arc::new(context)))
    }

    fn upload_grid(
        &self,
        part: ContextPart,
        grid: ContextGrid,
        samples: &[f32],
    ) -> PipelineResult<Texture> {
        check_texture(&self.ctx.device.limits(), "source context", grid.dims)?;
        let format = match part {
            ContextPart::Haze => MOMENT_FORMAT,
            ContextPart::Guide => CLARITY_GUIDE_FORMAT,
            ContextPart::Texture | ContextPart::Shadows => self.ctx.linear_format,
        };
        let texture = texture_2d(
            &self.ctx.device,
            "source-context",
            format,
            grid.dims,
            1,
            TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        );
        write_texture_2d(
            &self.ctx.queue,
            &texture,
            &grid_bytes(samples, part.channels(), format),
            grid.dims.0 * texel_size(format),
            grid.dims,
        );
        Ok(texture)
    }

    #[cfg(feature = "native")]
    pub(in crate::gpu::renderer) fn read_context(
        &self,
        context: &GlobalContext,
        haze_level: u32,
        cancel: Option<&CancelToken>,
    ) -> PipelineResult<ContextPlanes> {
        let haze = context.haze.as_ref().ok_or_else(|| {
            PipelineError::Render("a tile source context has no haze grid".into())
        })?;
        let levels = [
            (ContextPart::Texture, &context.presence.texture),
            (ContextPart::Guide, &context.presence.guide),
            (ContextPart::Shadows, &context.shadows),
        ];
        let stored = levels.into_iter().filter_map(|(part, view)| {
            view.as_ref()
                .map(|level| (part, &level.grid.view, level.level))
        });
        let planes = std::iter::once((ContextPart::Haze, &haze.ab.view, haze_level))
            .chain(stored)
            .map(|(part, view, level)| {
                let texture = view.texture();
                let size = texture.size();
                let dims = (size.width, size.height);
                let format = texture.format();
                let texels = dims.0 as usize * dims.1 as usize;
                let mut samples = Vec::with_capacity(texels * part.channels());
                self.read_texels(texture, Rect::full(dims), cancel, |row| {
                    extend_channels(&mut samples, row, format, part.channels())
                })?;
                Ok(ContextPlane {
                    part,
                    level,
                    dims,
                    samples,
                })
            })
            .collect::<PipelineResult<Vec<_>>>()?;
        Ok(ContextPlanes {
            atmosphere: haze.atmosphere,
            planes,
        })
    }
}

fn grid_bytes(samples: &[f32], channels: usize, format: TextureFormat) -> Vec<u8> {
    let texel = |px: &[f32]| [0, 1, 2, 3].map(|c| px.get(c).copied().unwrap_or(0.0));
    let texels = samples.chunks_exact(channels).map(texel);
    match format {
        TextureFormat::Rgba32Float => texels
            .flat_map(|rgba| rgba.into_iter().flat_map(f32::to_le_bytes))
            .collect(),
        _ => texels
            .flat_map(|rgba| {
                rgba.into_iter()
                    .flat_map(|v| f16::from_f32(v).to_bits().to_le_bytes())
            })
            .collect(),
    }
}

#[cfg(feature = "native")]
fn extend_channels(out: &mut Vec<f32>, row: &[u8], format: TextureFormat, channels: usize) {
    match format {
        TextureFormat::Rgba32Float => out.extend(row.as_chunks::<16>().0.iter().flat_map(|px| {
            px.as_chunks::<4>()
                .0
                .iter()
                .take(channels)
                .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        })),
        _ => out.extend(row.as_chunks::<8>().0.iter().flat_map(|px| {
            px.as_chunks::<2>()
                .0
                .iter()
                .take(channels)
                .map(|b| f16::from_bits(u16::from_le_bytes([b[0], b[1]])).to_f32())
        })),
    }
}

#[cfg(all(test, feature = "native"))]
mod tests {
    use super::*;

    #[test]
    fn grid_samples_round_trip_through_both_formats() {
        let samples: Vec<f32> = (0..12)
            .map(|i| f16::from_f32(i as f32 * 0.37).to_f32())
            .collect();
        for format in [TextureFormat::Rgba16Float, TextureFormat::Rgba32Float] {
            for channels in [1, 2] {
                let bytes = grid_bytes(&samples, channels, format);
                let mut back = Vec::new();
                extend_channels(&mut back, &bytes, format, channels);
                if back != samples {
                    panic!("{format:?} with {channels} channels changed the samples");
                }
            }
        }
    }
}
