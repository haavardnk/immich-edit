use std::sync::Arc;

use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindingResource, BufferUsages, CommandEncoderDescriptor,
    ComputePassDescriptor, Extent3d, Texture, TextureDescriptor, TextureDimension, TextureUsages,
    TextureViewDescriptor,
};

use crate::PipelineResult;
use crate::edits::{Edits, RetouchMode, RetouchStroke};
use crate::frame::RawFrame;
use crate::gpu::helpers::mip_count;
use crate::gpu::passes::retouch::RETOUCH_UNIFORM_SIZE;
use crate::ops::retouch::{StrokeGeom, stroke_geometry};

use super::GpuRenderer;

fn write_uniform(geom: &StrokeGeom, stroke: &RetouchStroke, dims: (u32, u32), dir: u32) -> Vec<u8> {
    let mut b = vec![0u8; RETOUCH_UNIFORM_SIZE as usize];
    let bw = (geom.bbox.x1 - geom.bbox.x0) as u32;
    let bh = (geom.bbox.y1 - geom.bbox.y0) as u32;
    let heal = matches!(stroke.mode, RetouchMode::Heal);
    b[0..4].copy_from_slice(&dims.0.to_le_bytes());
    b[4..8].copy_from_slice(&dims.1.to_le_bytes());
    b[8..12].copy_from_slice(&(geom.bbox.x0 as u32).to_le_bytes());
    b[12..16].copy_from_slice(&(geom.bbox.y0 as u32).to_le_bytes());
    b[16..20].copy_from_slice(&bw.to_le_bytes());
    b[20..24].copy_from_slice(&bh.to_le_bytes());
    b[24..28].copy_from_slice(&(geom.points.len() as u32).to_le_bytes());
    b[28..32].copy_from_slice(&u32::from(!heal).to_le_bytes());
    b[32..36].copy_from_slice(&geom.off_x.to_le_bytes());
    b[36..40].copy_from_slice(&geom.off_y.to_le_bytes());
    b[40..44].copy_from_slice(&geom.radius_px.to_le_bytes());
    b[44..48].copy_from_slice(&stroke.hardness.to_le_bytes());
    b[48..52].copy_from_slice(&stroke.opacity.to_le_bytes());
    b[52..56].copy_from_slice(&geom.sigma.to_le_bytes());
    b[56..60].copy_from_slice(&dir.to_le_bytes());
    b
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
                &write_uniform(&geom, stroke, dims, 0),
                "retouch-prep-u",
            );
            let blur_h_u = self.uniform_pool.acquire(
                device,
                queue,
                &write_uniform(&geom, stroke, dims, 0),
                "retouch-blur-h-u",
            );
            let blur_v_u = self.uniform_pool.acquire(
                device,
                queue,
                &write_uniform(&geom, stroke, dims, 1),
                "retouch-blur-v-u",
            );

            let prep_bind = device.create_bind_group(&BindGroupDescriptor {
                label: Some("retouch-prep-bg"),
                layout: &p.prep_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: prep_u.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(&src_view),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::TextureView(&patch_src_view),
                    },
                    BindGroupEntry {
                        binding: 3,
                        resource: BindingResource::TextureView(&patch_res_view),
                    },
                ],
            });
            let blur_h_bind = device.create_bind_group(&BindGroupDescriptor {
                label: Some("retouch-blur-h-bg"),
                layout: &p.blur_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: blur_h_u.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(&patch_res_view),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::TextureView(&patch_tmp_view),
                    },
                ],
            });
            let blur_v_bind = device.create_bind_group(&BindGroupDescriptor {
                label: Some("retouch-blur-v-bg"),
                layout: &p.blur_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: blur_v_u.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(&patch_tmp_view),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::TextureView(&patch_res_view),
                    },
                ],
            });
            let apply_bind = device.create_bind_group(&BindGroupDescriptor {
                label: Some("retouch-apply-bg"),
                layout: &p.apply_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: prep_u.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(&src_view),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::TextureView(&patch_src_view),
                    },
                    BindGroupEntry {
                        binding: 3,
                        resource: BindingResource::TextureView(&patch_res_view),
                    },
                    BindGroupEntry {
                        binding: 4,
                        resource: pts_buf.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 5,
                        resource: BindingResource::TextureView(&dst_mip0),
                    },
                ],
            });

            let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
                label: Some("retouch-enc"),
            });
            {
                let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                    label: Some("retouch-prep"),
                    timestamp_writes: None,
                });
                cpass.set_pipeline(&p.prep_pipeline);
                cpass.set_bind_group(0, &prep_bind, &[]);
                cpass.dispatch_workgroups(bw.div_ceil(16), bh.div_ceil(16), 1);
            }
            if heal {
                let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                    label: Some("retouch-blur-h"),
                    timestamp_writes: None,
                });
                cpass.set_pipeline(&p.blur_pipeline);
                cpass.set_bind_group(0, &blur_h_bind, &[]);
                cpass.dispatch_workgroups(bw.div_ceil(16), bh.div_ceil(16), 1);
                cpass.set_bind_group(0, &blur_v_bind, &[]);
                cpass.dispatch_workgroups(bw.div_ceil(16), bh.div_ceil(16), 1);
            }
            {
                let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                    label: Some("retouch-apply"),
                    timestamp_writes: None,
                });
                cpass.set_pipeline(&p.apply_pipeline);
                cpass.set_bind_group(0, &apply_bind, &[]);
                cpass.dispatch_workgroups(w.div_ceil(16), h.div_ceil(16), 1);
            }
            self.encode_mipgen(&mut encoder, &dst, w, h);
            queue.submit(Some(encoder.finish()));
            current = Arc::new(dst);
        }

        Ok(current)
    }
}
