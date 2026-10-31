use std::sync::Arc;

use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BufferUsages, CommandEncoder, CommandEncoderDescriptor, Texture, TextureUsages, TextureView,
};

use crate::PipelineResult;
use crate::edits::{Edits, RetouchMode, RetouchStroke};
use crate::frame::OrientFlips;
use crate::gpu::dispatch::{Region, bind_group, copy_region, dispatch_2d, tex};
use crate::gpu::helpers::mip_count;
use crate::gpu::passes::retouch::RetouchParams;
use crate::gpu::texture::{STORAGE_SAMPLED, full_view, mip_view, texture_2d};
use crate::ops::retouch::StrokeGeom;
use crate::tile::{Rect, SensorTile, tile_strokes};

use super::GpuRenderer;

fn retouch_params(
    geom: &StrokeGeom,
    stroke: &RetouchStroke,
    sensor: (u32, u32),
    window: Rect,
) -> RetouchParams {
    let heal = matches!(stroke.mode, RetouchMode::Heal);
    RetouchParams {
        dims: [sensor.0, sensor.1],
        origin: [window.x, window.y],
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
        _pad: 0.0,
    }
}

impl GpuRenderer {
    pub(super) fn submit_retouch(
        &self,
        src: Arc<Texture>,
        tile: &SensorTile,
        sensor: (u32, u32),
        orientation: OrientFlips,
        edits: &Edits,
    ) -> PipelineResult<Arc<Texture>> {
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let window = tile.window();
        let (w, h) = window.dims();
        let p = &self.passes.sensor_stage.retouch;
        let strokes = tile_strokes(edits, tile, sensor, orientation);
        let mut current = src;

        for (i, (stroke, geom)) in strokes.iter().enumerate() {
            let bw = (geom.bbox.x1 - geom.bbox.x0) as u32;
            let bh = (geom.bbox.y1 - geom.bbox.y0) as u32;
            let heal = matches!(stroke.mode, RetouchMode::Heal);

            let pts: Vec<f32> = geom.points.iter().flat_map(|p| [p.0, p.1]).collect();
            let pts_buf = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("retouch-points"),
                contents: bytemuck::cast_slice(&pts),
                usage: BufferUsages::STORAGE,
            });

            let make_patch = |label: &'static str, levels: u32| {
                texture_2d(
                    device,
                    label,
                    self.ctx.linear_format,
                    (bw, bh),
                    levels,
                    STORAGE_SAMPLED,
                )
            };
            let levels = if heal { mip_count(bw, bh) } else { 1 };
            let patch_src = make_patch("retouch-patch-src", 1);
            let pyramid = make_patch("retouch-heal-pyramid", levels);
            let fill = make_patch("retouch-heal-fill", levels);
            let patch_src_view = full_view(&patch_src);
            let pyramid_views: Vec<TextureView> =
                (0..levels).map(|l| mip_view(&pyramid, l)).collect();
            let fill_views: Vec<TextureView> = (0..levels).map(|l| mip_view(&fill, l)).collect();

            let out_levels = if i + 1 == strokes.len() && !tile.grown() {
                mip_count(w, h)
            } else {
                1
            };
            let dst = texture_2d(
                device,
                "retouch-out",
                self.ctx.linear_format,
                (w, h),
                out_levels,
                STORAGE_SAMPLED | TextureUsages::COPY_SRC,
            );
            let src_view = full_view(&current);
            let dst_mip0 = mip_view(&dst, 0);

            let params = self.uniform(&retouch_params(geom, stroke, sensor, window), "retouch-u");

            let prep_bind = bind_group(
                device,
                "retouch-prep-bg",
                &p.prep_layout,
                &[
                    params.as_entire_binding(),
                    tex(&src_view),
                    tex(&patch_src_view),
                    tex(&pyramid_views[0]),
                    pts_buf.as_entire_binding(),
                ],
            );
            let apply_bind = bind_group(
                device,
                "retouch-apply-bg",
                &p.apply_layout,
                &[
                    params.as_entire_binding(),
                    tex(&src_view),
                    tex(&patch_src_view),
                    tex(&fill_views[0]),
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
                self.encode_heal_fill(&mut encoder, &pyramid_views, &fill_views, (bw, bh));
            }
            dispatch_2d(
                &mut encoder,
                "retouch-apply",
                &p.apply_pipeline,
                &apply_bind,
                w.div_ceil(16),
                h.div_ceil(16),
            );
            if out_levels > 1 {
                self.encode_mipgen(&mut encoder, &dst, w, h);
            }
            queue.submit(Some(encoder.finish()));
            current = Arc::new(dst);
        }

        if !tile.grown() {
            return Ok(current);
        }
        Ok(self.submit_crop(&current, tile.detail.within(window)))
    }

    fn submit_crop(&self, src: &Texture, rect: Rect) -> Arc<Texture> {
        let (w, h) = rect.dims();
        let out = texture_2d(
            &self.ctx.device,
            "retouch-crop",
            self.ctx.linear_format,
            (w, h),
            mip_count(w, h),
            STORAGE_SAMPLED | TextureUsages::COPY_DST | TextureUsages::COPY_SRC,
        );
        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("retouch-crop"),
            });
        copy_region(
            &mut encoder,
            Region::at(src, 0, (rect.x, rect.y)),
            Region::at(&out, 0, (0, 0)),
            (w, h),
        );
        self.encode_mipgen(&mut encoder, &out, w, h);
        self.ctx.queue.submit(Some(encoder.finish()));
        Arc::new(out)
    }

    fn encode_heal_fill(
        &self,
        encoder: &mut CommandEncoder,
        pyramid: &[TextureView],
        fill: &[TextureView],
        size: (u32, u32),
    ) {
        let device = &self.ctx.device;
        let p = &self.passes.sensor_stage.retouch;
        let groups = |level: usize| {
            (
                (size.0 >> level).max(1).div_ceil(16),
                (size.1 >> level).max(1).div_ceil(16),
            )
        };
        for level in 1..pyramid.len() {
            let bind = bind_group(
                device,
                "retouch-push-bg",
                &p.push_layout,
                &[tex(&pyramid[level - 1]), tex(&pyramid[level])],
            );
            let (gx, gy) = groups(level);
            dispatch_2d(encoder, "retouch-push", &p.push_pipeline, &bind, gx, gy);
        }
        let top = pyramid.len() - 1;
        for level in (0..pyramid.len()).rev() {
            let coarse = if level == top {
                &pyramid[top]
            } else {
                &fill[level + 1]
            };
            let bind = bind_group(
                device,
                "retouch-pull-bg",
                &p.pull_layout,
                &[tex(&pyramid[level]), tex(coarse), tex(&fill[level])],
            );
            let (gx, gy) = groups(level);
            dispatch_2d(encoder, "retouch-pull", &p.pull_pipeline, &bind, gx, gy);
        }
    }
}
