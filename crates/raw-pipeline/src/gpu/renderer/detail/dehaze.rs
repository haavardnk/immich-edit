use wgpu::{CommandEncoderDescriptor, Texture};

use crate::gpu::dispatch::{begin_pass, bind_group, tex};
use crate::gpu::passes::dehaze::{
    DehazeApplyParams, DehazeDownsampleParams, DehazeFilterParams, DehazeNormParams,
    DehazeSizeParams, MOMENT_FORMAT,
};
use crate::gpu::renderer::GpuRenderer;
use crate::gpu::renderer::display::{GridView, HazeGrid, Placed};
use crate::gpu::texture::{STORAGE_SAMPLED, STORAGE_SAMPLED_SRC, full_view};
use crate::gpu::texture_pool::{PooledTexture, TextureKey};
use crate::ops::dehaze::DehazeGrid;
use crate::tile::Rect;

pub(in crate::gpu::renderer) fn haze_dims((w, h): (u32, u32), scale: u32) -> (u32, u32) {
    ((w / scale).max(1), (h / scale).max(1))
}

#[derive(Clone, Copy)]
pub(in crate::gpu::renderer) struct HazeSample {
    pub cells: Rect,
    pub read: (u32, u32),
    pub scale: u32,
}

impl GpuRenderer {
    pub(in crate::gpu::renderer) fn submit_local_haze(
        &self,
        src: &Texture,
        placed: Placed,
        atmosphere: [f32; 3],
    ) -> (HazeGrid, PooledTexture) {
        let Placed { extent, origin } = placed;
        let grid = DehazeGrid::for_dims(extent.full);
        let lo = haze_dims(extent.dims, grid.scale);
        let lo_src = self.texture_pool.acquire(
            &self.ctx.device,
            TextureKey::new(self.ctx.linear_format, lo.0, lo.1, 1, STORAGE_SAMPLED),
            "dehaze-lo-src",
        );
        let sample = HazeSample {
            cells: Rect::full(lo),
            read: (0, 0),
            scale: grid.scale,
        };
        self.submit_haze_downsample(src, &lo_src, sample);
        let ab = self.submit_haze_grid(&lo_src, lo, grid, atmosphere);
        let haze = HazeGrid {
            atmosphere,
            ab: GridView::local(full_view(&ab), origin, grid.scale.trailing_zeros()),
        };
        (haze, ab)
    }

    pub(in crate::gpu::renderer) fn submit_haze_downsample(
        &self,
        src: &Texture,
        dst: &Texture,
        sample: HazeSample,
    ) {
        let device = &self.ctx.device;
        let HazeSample { cells, read, scale } = sample;
        let uniform = self.uniform(
            &DehazeDownsampleParams {
                size: [cells.w, cells.h],
                read: [read.0, read.1],
                write: [cells.x, cells.y],
                scale,
                _pad: 0,
            },
            "dehaze-downsample-u",
        );
        let p = &self.passes.dehaze;
        let bind = bind_group(
            device,
            "dehaze-downsample-bg",
            &p.downsample_layout,
            &[
                uniform.as_entire_binding(),
                tex(&full_view(src)),
                tex(&full_view(dst)),
            ],
        );
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("dehaze-downsample-enc"),
        });
        {
            let mut c = begin_pass(&mut encoder, "dehaze-downsample-pass");
            c.set_pipeline(&p.downsample_pipeline);
            c.set_bind_group(0, &bind, &[]);
            c.dispatch_workgroups(cells.w.div_ceil(16), cells.h.div_ceil(16), 1);
        }
        self.ctx.queue.submit(Some(encoder.finish()));
    }

    pub(in crate::gpu::renderer) fn submit_haze_grid(
        &self,
        lo_src: &Texture,
        (lw, lh): (u32, u32),
        grid: DehazeGrid,
        atm: [f32; 3],
    ) -> PooledTexture {
        let device = &self.ctx.device;
        let moment_key = TextureKey::new(MOMENT_FORMAT, lw, lh, 1, STORAGE_SAMPLED_SRC);
        let moments = [
            self.texture_pool
                .acquire(device, moment_key, "dehaze-moments-a"),
            self.texture_pool
                .acquire(device, moment_key, "dehaze-moments-b"),
        ];
        let norm_buf = self.uniform(
            &DehazeNormParams {
                size: [lw, lh],
                _pad: [0; 2],
                atmosphere: [atm[0], atm[1], atm[2], 1.0],
            },
            "dehaze-norm-u",
        );
        let filter_u = |radius: u32, axis: u32, label: &'static str| {
            let params = DehazeFilterParams {
                size: [lw, lh],
                radius,
                axis,
            };
            self.uniform(&params, label)
        };
        let min_h_buf = filter_u(grid.patch, 0, "dehaze-min-h-u");
        let min_v_buf = filter_u(grid.patch, 1, "dehaze-min-v-u");
        let box_h_buf = filter_u(grid.guided, 0, "dehaze-box-h-u");
        let box_v_buf = filter_u(grid.guided, 1, "dehaze-box-v-u");
        let size_buf = self.uniform(
            &DehazeSizeParams {
                size: [lw, lh],
                _pad: [0; 2],
            },
            "dehaze-lo-size-u",
        );
        let lo_view = full_view(lo_src);
        let a = full_view(&moments[0]);
        let b = full_view(&moments[1]);
        let p = &self.passes.dehaze;
        let steps = [
            (&p.norm_pipeline, &p.norm_layout, &norm_buf, None, &a),
            (&p.min_pipeline, &p.min_layout, &min_h_buf, None, &b),
            (&p.min_pipeline, &p.min_layout, &min_v_buf, None, &a),
            (&p.pack_pipeline, &p.pack_layout, &size_buf, Some(&a), &b),
            (&p.box_pipeline, &p.box_layout, &box_h_buf, None, &a),
            (&p.box_pipeline, &p.box_layout, &box_v_buf, None, &b),
            (&p.ab_pipeline, &p.ab_layout, &size_buf, None, &a),
            (&p.box_pipeline, &p.box_layout, &box_h_buf, None, &b),
            (&p.box_pipeline, &p.box_layout, &box_v_buf, None, &a),
        ];
        let mut input = &lo_view;
        let binds: Vec<_> = steps
            .iter()
            .map(|&(pipeline, layout, uniform, extra, output)| {
                let bind = match extra {
                    Some(extra) => bind_group(
                        device,
                        "dehaze-grid-bg",
                        layout,
                        &[
                            uniform.as_entire_binding(),
                            tex(&lo_view),
                            tex(extra),
                            tex(output),
                        ],
                    ),
                    None => bind_group(
                        device,
                        "dehaze-grid-bg",
                        layout,
                        &[uniform.as_entire_binding(), tex(input), tex(output)],
                    ),
                };
                input = output;
                (pipeline, bind)
            })
            .collect();
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("dehaze-grid-enc"),
        });
        {
            let mut c = begin_pass(&mut encoder, "dehaze-grid-pass");
            for (pipeline, bind) in &binds {
                c.set_pipeline(pipeline);
                c.set_bind_group(0, bind, &[]);
                c.dispatch_workgroups(lw.div_ceil(16), lh.div_ceil(16), 1);
            }
        }
        self.ctx.queue.submit(Some(encoder.finish()));
        let [ab, _] = moments;
        ab
    }

    pub(in crate::gpu::renderer) fn submit_dehaze_apply(
        &self,
        src: &Texture,
        placed: Placed,
        haze: &HazeGrid,
        amount: f32,
    ) -> PooledTexture {
        let device = &self.ctx.device;
        let Placed { extent, origin } = placed;
        let (w, h) = extent.dims;
        let scale = DehazeGrid::for_dims(extent.full).scale;
        let lo = haze.ab.view.texture().size();
        let atm = haze.atmosphere;
        let uniform = self.uniform(
            &DehazeApplyParams {
                size: [w, h],
                lo_size: [lo.width, lo.height],
                origin: [origin.0, origin.1],
                grid_origin: [haze.ab.origin.0, haze.ab.origin.1],
                atmosphere: [atm[0], atm[1], atm[2], 1.0],
                amount,
                scale: scale as f32,
                _pad: [0.0; 2],
            },
            "dehaze-apply-u",
        );
        let out = self.texture_pool.acquire(
            device,
            TextureKey::new(self.ctx.linear_format, w, h, 1, STORAGE_SAMPLED),
            "dehaze-out",
        );
        let p = &self.passes.dehaze;
        let bind = bind_group(
            device,
            "dehaze-apply-bg",
            &p.apply_layout,
            &[
                uniform.as_entire_binding(),
                tex(&full_view(src)),
                tex(&haze.ab.view),
                tex(&full_view(&out)),
            ],
        );
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("dehaze-apply-enc"),
        });
        {
            let mut c = begin_pass(&mut encoder, "dehaze-apply-pass");
            c.set_pipeline(&p.apply_pipeline);
            c.set_bind_group(0, &bind, &[]);
            c.dispatch_workgroups(w.div_ceil(16), h.div_ceil(16), 1);
        }
        self.ctx.queue.submit(Some(encoder.finish()));
        out
    }
}
