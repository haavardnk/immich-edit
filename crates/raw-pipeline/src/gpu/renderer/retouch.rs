use std::sync::Arc;

use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BufferUsages, CommandEncoderDescriptor, ComputePassDescriptor, Extent3d, Texture,
    TextureDescriptor, TextureDimension, TextureUsages, TextureViewDescriptor,
};

use crate::PipelineResult;
use crate::edits::{Edits, RetouchMode, RetouchStroke};
use crate::frame::RawFrame;
use crate::gpu::dispatch::{bind_group, dispatch_2d, tex};
use crate::gpu::helpers::mip_count;
use crate::gpu::passes::retouch::RetouchParams;
use crate::ops::retouch::{StrokeGeom, stroke_geometry};

use super::GpuRenderer;

fn retouch_params(
    geom: &StrokeGeom,
    stroke: &RetouchStroke,
    dims: (u32, u32),
    dir: u32,
) -> RetouchParams {
    let heal = matches!(stroke.mode, RetouchMode::Heal);
    RetouchParams {
        dims: [dims.0, dims.1],
        bbox_origin: [geom.bbox.x0 as u32, geom.bbox.y0 as u32],
        bbox_size: [
            (geom.bbox.x1 - geom.bbox.x0) as u32,
            (geom.bbox.y1 - geom.bbox.y0) as u32,
        ],
        point_count: geom.points.len() as u32,
        clone_mode: u32::from(!heal),
        offset: [geom.off_x, geom.off_y],
        radius_px: geom.radius_px,
        hardness: stroke.hardness,
        opacity: stroke.opacity,
        sigma: geom.sigma,
        dir,
        _pad: 0.0,
    }
}

impl GpuRenderer {
    pub(super) fn run_retouch(
        &self,
        src: Arc<Texture>,
        dims: (u32, u32),
        frame: &RawFrame,
        edits: &Edits,
    ) -> PipelineResult<Arc<Texture>> {
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let (w, h) = dims;
        let p = &self.passes.retouch;
        let mut current = src;

        for stroke in edits.retouch.iter().filter(|s| s.is_effective()) {
            let Some(geom) = stroke_geometry(stroke, w as usize, h as usize, frame.orientation)
            else {
                continue;
            };
            let bw = (geom.bbox.x1 - geom.bbox.x0) as u32;
            let bh = (geom.bbox.y1 - geom.bbox.y0) as u32;
            let heal = matches!(stroke.mode, RetouchMode::Heal);

            let pts: Vec<f32> = geom.points.iter().flat_map(|p| [p.0, p.1]).collect();
            let pts_buf = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("retouch-points"),
                contents: bytemuck::cast_slice(&pts),
                usage: BufferUsages::STORAGE,
            });

            let make_patch = |label: &'static str| {
                device.create_texture(&TextureDescriptor {
                    label: Some(label),
                    size: Extent3d {
                        width: bw,
                        height: bh,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format: self.ctx.linear_format,
                    usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                })
            };
            let patch_src = make_patch("retouch-patch-src");
            let patch_res = make_patch("retouch-patch-res");
            let patch_tmp = make_patch("retouch-patch-tmp");
            let patch_src_view = patch_src.create_view(&TextureViewDescriptor::default());
            let patch_res_view = patch_res.create_view(&TextureViewDescriptor::default());
            let patch_tmp_view = patch_tmp.create_view(&TextureViewDescriptor::default());

            let dst = device.create_texture(&TextureDescriptor {
                label: Some("retouch-out"),
                size: Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: mip_count(w, h),
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: self.ctx.linear_format,
                usage: TextureUsages::STORAGE_BINDING
                    | TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let src_view = current.create_view(&TextureViewDescriptor::default());
            let dst_mip0 = dst.create_view(&TextureViewDescriptor {
                base_mip_level: 0,
                mip_level_count: Some(1),
                ..Default::default()
            });

            let prep_u = self.uniform_pool.acquire(
                device,
                queue,
                bytemuck::bytes_of(&retouch_params(&geom, stroke, dims, 0)),
                "retouch-prep-u",
            );
            let blur_h_u = self.uniform_pool.acquire(
                device,
                queue,
                bytemuck::bytes_of(&retouch_params(&geom, stroke, dims, 0)),
                "retouch-blur-h-u",
            );
            let blur_v_u = self.uniform_pool.acquire(
                device,
                queue,
                bytemuck::bytes_of(&retouch_params(&geom, stroke, dims, 1)),
                "retouch-blur-v-u",
            );

            let prep_bind = bind_group(
                device,
                "retouch-prep-bg",
                &p.prep_layout,
                &[
                    prep_u.as_entire_binding(),
                    tex(&src_view),
                    tex(&patch_src_view),
                    tex(&patch_res_view),
                ],
            );
            let blur_h_bind = bind_group(
                device,
                "retouch-blur-h-bg",
                &p.blur_layout,
                &[
                    blur_h_u.as_entire_binding(),
                    tex(&patch_res_view),
                    tex(&patch_tmp_view),
                ],
            );
            let blur_v_bind = bind_group(
                device,
                "retouch-blur-v-bg",
                &p.blur_layout,
                &[
                    blur_v_u.as_entire_binding(),
                    tex(&patch_tmp_view),
                    tex(&patch_res_view),
                ],
            );
            let apply_bind = bind_group(
                device,
                "retouch-apply-bg",
                &p.apply_layout,
                &[
                    prep_u.as_entire_binding(),
                    tex(&src_view),
                    tex(&patch_src_view),
                    tex(&patch_res_view),
                    pts_buf.as_entire_binding(),
                    tex(&dst_mip0),
                ],
            );

            let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
                label: Some("retouch-enc"),
            });
            dispatch_2d(
                &mut encoder,
                "retouch-prep",
                &p.prep_pipeline,
                &prep_bind,
                bw.div_ceil(16),
                bh.div_ceil(16),
            );
            if heal {
                let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                    label: Some("retouch-blur"),
                    timestamp_writes: None,
                });
                cpass.set_pipeline(&p.blur_pipeline);
                cpass.set_bind_group(0, &blur_h_bind, &[]);
                cpass.dispatch_workgroups(bw.div_ceil(16), bh.div_ceil(16), 1);
                cpass.set_bind_group(0, &blur_v_bind, &[]);
                cpass.dispatch_workgroups(bw.div_ceil(16), bh.div_ceil(16), 1);
            }
            dispatch_2d(
                &mut encoder,
                "retouch-apply",
                &p.apply_pipeline,
                &apply_bind,
                w.div_ceil(16),
                h.div_ceil(16),
            );
            self.encode_mipgen(&mut encoder, &dst, w, h);
            queue.submit(Some(encoder.finish()));
            current = Arc::new(dst);
        }

        Ok(current)
    }
}
