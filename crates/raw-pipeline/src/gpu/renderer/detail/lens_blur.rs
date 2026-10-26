use std::sync::Arc;

use wgpu::{CommandEncoderDescriptor, Texture, TextureUsages};

use crate::PipelineResult;
use crate::edits::LensEdits;
use crate::frame::OrientFlips;
use crate::gpu::dispatch::{begin_pass, bind_group, tex};
use crate::gpu::passes::lens_blur::{DEPTH_FORMAT, GRID_FORMAT, LensBlurParams};
use crate::gpu::renderer::GpuRenderer;
use crate::gpu::source::SourceExtent;
use crate::gpu::texture::{STORAGE_SAMPLED, full_view, texture_2d, write_texture_2d};
use crate::gpu::texture_pool::{PooledTexture, TextureKey};
use crate::ops::lens_blur::LensBlur;
use crate::ops::lens_distortion::LensWarpParams;

pub(in crate::gpu::renderer) struct LensBlurInput<'a> {
    pub extent: SourceExtent,
    pub origin: (u32, u32),
    pub orientation: OrientFlips,
    pub blur: &'a LensBlur,
    pub lens: &'a LensEdits,
}

#[derive(Debug, PartialEq, Eq)]
struct GridWindow {
    origin: [u32; 2],
    size: [u32; 2],
}

fn grid_window(
    origin: (u32, u32),
    dims: (u32, u32),
    full: (u32, u32),
    orientation: OrientFlips,
    ds: u32,
) -> GridWindow {
    let span = |start: u32, len: u32, total: u32, flip: bool| {
        if flip {
            (total - start - len, total - start)
        } else {
            (start, start + len)
        }
    };
    let xs = span(origin.0, dims.0, full.0, orientation.1);
    let ys = span(origin.1, dims.1, full.1, orientation.2);
    let (xs, ys) = if orientation.0 { (ys, xs) } else { (xs, ys) };
    let cells = |(start, end): (u32, u32)| (start / ds, end.div_ceil(ds) - start / ds);
    let (gx, gw) = cells(xs);
    let (gy, gh) = cells(ys);
    GridWindow {
        origin: [gx, gy],
        size: [gw, gh],
    }
}

impl GpuRenderer {
    fn depth_texture(&self, blur: &LensBlur) -> Arc<Texture> {
        if let Some(texture) = self.depth_tex_cache.lock().get(&blur.raster_id) {
            return texture.clone();
        }
        let raster = &blur.depth;
        let dims = (raster.width, raster.height);
        let texture = Arc::new(texture_2d(
            &self.ctx.device,
            "lens-blur-depth",
            DEPTH_FORMAT,
            dims,
            1,
            TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        ));
        write_texture_2d(&self.ctx.queue, &texture, &raster.bytes, raster.width, dims);
        self.depth_tex_cache
            .lock()
            .put(blur.raster_id.clone(), texture.clone());
        texture
    }

    pub(in crate::gpu::renderer) fn submit_lens_blur(
        &self,
        src: &Texture,
        input: LensBlurInput<'_>,
    ) -> PipelineResult<PooledTexture> {
        let LensBlurInput {
            extent,
            origin,
            orientation,
            blur,
            lens,
        } = input;
        let device = &self.ctx.device;
        let (w, h) = extent.dims;
        let full = extent.full;
        let ds = blur.scale.ds;
        let grid = grid_window(origin, extent.dims, full, orientation, ds);
        let oriented = if orientation.0 {
            (full.1, full.0)
        } else {
            full
        };
        let warp = LensWarpParams::from_edits(lens, oriented.0, oriented.1);
        let params = LensBlurParams {
            size: [w, h],
            origin: [origin.0, origin.1],
            full: [full.0, full.1],
            grid_origin: grid.origin,
            grid_size: grid.size,
            ds,
            reach: blur.scale.reach,
            orient: [
                u32::from(orientation.0),
                u32::from(orientation.1),
                u32::from(orientation.2),
            ],
            _pad0: 0,
            lens: [warp.k1, warp.k2, warp.k3, warp.zoom],
            radius: blur.scale.radius,
            boost: blur.boost,
            blades: blur.blades,
            _pad: 0,
            table: bytemuck::cast(blur.table.values),
        };
        let uniform = self.uniform(&params, "lens-blur-uniform");
        let depth = self.depth_texture(blur);
        let grid_key = TextureKey::new(GRID_FORMAT, grid.size[0], grid.size[1], 1, STORAGE_SAMPLED);
        let cells = self
            .texture_pool
            .acquire(device, grid_key, "lens-blur-grid");
        let gathered = self
            .texture_pool
            .acquire(device, grid_key, "lens-blur-gathered");
        let out = self.texture_pool.acquire(
            device,
            TextureKey::new(self.ctx.linear_format, w, h, 1, STORAGE_SAMPLED),
            "lens-blur-out",
        );
        let src_view = full_view(src);
        let depth_view = full_view(&depth);
        let cells_view = full_view(&cells);
        let gathered_view = full_view(&gathered);
        let out_view = full_view(&out);
        let passes = &self.passes.lens_blur;
        let down = bind_group(
            device,
            "lens-blur-down-bg",
            &passes.down_layout,
            &[
                uniform.as_entire_binding(),
                tex(&src_view),
                tex(&depth_view),
                tex(&cells_view),
            ],
        );
        let gather = bind_group(
            device,
            "lens-blur-gather-bg",
            &passes.gather_layout,
            &[
                uniform.as_entire_binding(),
                tex(&cells_view),
                tex(&gathered_view),
            ],
        );
        let apply = bind_group(
            device,
            "lens-blur-apply-bg",
            &passes.apply_layout,
            &[
                uniform.as_entire_binding(),
                tex(&src_view),
                tex(&depth_view),
                tex(&cells_view),
                tex(&gathered_view),
                tex(&out_view),
            ],
        );
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("lens-blur-enc"),
        });
        {
            let mut pass = begin_pass(&mut encoder, "lens-blur-pass");
            let (gx, gy) = (grid.size[0].div_ceil(16), grid.size[1].div_ceil(16));
            pass.set_pipeline(&passes.down_pipeline);
            pass.set_bind_group(0, &down, &[]);
            pass.dispatch_workgroups(gx, gy, 1);
            pass.set_pipeline(&passes.gather_pipeline);
            pass.set_bind_group(0, &gather, &[]);
            pass.dispatch_workgroups(gx, gy, 1);
            pass.set_pipeline(&passes.apply_pipeline);
            pass.set_bind_group(0, &apply, &[]);
            pass.dispatch_workgroups(w.div_ceil(16), h.div_ceil(16), 1);
        }
        self.ctx.queue.submit(Some(encoder.finish()));
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_full_frame_covers_every_oriented_cell() {
        let grid = grid_window((0, 0), (1203, 805), (1203, 805), (true, true, false), 4);
        assert_eq!(
            grid,
            GridWindow {
                origin: [0, 0],
                size: [202, 301],
            }
        );
    }

    #[test]
    fn a_flipped_window_maps_to_the_mirrored_cells() {
        let grid = grid_window((100, 40), (200, 60), (1000, 500), (false, true, true), 8);
        assert_eq!(
            grid,
            GridWindow {
                origin: [87, 50],
                size: [26, 8],
            }
        );
    }
}
