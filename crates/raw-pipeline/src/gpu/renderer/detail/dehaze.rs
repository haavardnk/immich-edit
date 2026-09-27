use wgpu::{CommandEncoderDescriptor, Texture};

use crate::PipelineResult;
use crate::edits::Edits;
use crate::gpu::dispatch::{begin_pass, bind_group, samp, tex};
use crate::gpu::passes::dehaze::{
    DehazeApplyParams, DehazeDownsampleParams, DehazeFilterParams, DehazeNormParams, MOMENT_FORMAT,
};
use crate::gpu::renderer::GpuRenderer;
use crate::gpu::source::SourceExtent;
use crate::gpu::texture::{STORAGE_SAMPLED, full_view, mip_view};
use crate::gpu::texture_pool::{PooledTexture, TextureKey};
use crate::ops::dehaze::DehazeGrid;

impl GpuRenderer {
    pub(in crate::gpu::renderer) fn submit_dehaze(
        &self,
        src: &Texture,
        extent: SourceExtent,
        edits: &Edits,
        atm: [f32; 3],
    ) -> PipelineResult<PooledTexture> {
        let device = &self.ctx.device;
        let (w, h) = extent.dims;
        let DehazeGrid {
            scale,
            patch: r_patch,
            guided: r_gf,
        } = DehazeGrid::for_dims(extent.full);
        let lw = (w / scale).max(1);
        let lh = (h / scale).max(1);
        let amount = (edits.basic.dehaze as f32 / 100.0).clamp(-1.0, 1.0);

        let scratch_key = TextureKey::new(self.ctx.linear_format, lw, lh, 1, STORAGE_SAMPLED);
        let moment_key = TextureKey::new(MOMENT_FORMAT, lw, lh, 1, STORAGE_SAMPLED);
        let make_scratch_lo =
            |label: &'static str| self.texture_pool.acquire(device, scratch_key, label);
        let make_moment_lo =
            |label: &'static str| self.texture_pool.acquire(device, moment_key, label);
        let lo_src = make_scratch_lo("dehaze-lo-src");
        let dn = make_moment_lo("dehaze-dn");
        let dn_h = make_moment_lo("dehaze-dn-h");
        let dn_min = make_moment_lo("dehaze-dn-min");
        let packed = make_moment_lo("dehaze-pack");
        let packed_h = make_moment_lo("dehaze-pack-h");
        let packed_v = make_moment_lo("dehaze-pack-v");
        let ab = make_moment_lo("dehaze-ab");
        let ab_h = make_moment_lo("dehaze-ab-h");
        let ab_v = make_moment_lo("dehaze-ab-v");
        let out_key = TextureKey::new(self.ctx.linear_format, w, h, 1, STORAGE_SAMPLED);
        let out = self.texture_pool.acquire(device, out_key, "dehaze-out");

        let downsample_buf = self.uniform(
            &DehazeDownsampleParams {
                size: [lw, lh],
                scale,
                _pad: 0,
            },
            "dehaze-downsample-u",
        );
        let norm_buf = self.uniform(
            &DehazeNormParams {
                size: [lw, lh],
                _pad: [0; 2],
                atmosphere: [atm[0], atm[1], atm[2], 1.0],
            },
            "dehaze-norm-u",
        );

        let make_filter_u = |radius: u32, axis: u32, label: &'static str| {
            let params = DehazeFilterParams {
                size: [lw, lh],
                radius,
                axis,
            };
            self.uniform(&params, label)
        };
        let min_h_buf = make_filter_u(r_patch, 0, "dehaze-min-h-u");
        let min_v_buf = make_filter_u(r_patch, 1, "dehaze-min-v-u");
        let box_h_buf = make_filter_u(r_gf, 0, "dehaze-box-h-u");
        let box_v_buf = make_filter_u(r_gf, 1, "dehaze-box-v-u");

        let pack_buf = make_filter_u(0, 0, "dehaze-pack-u");
        let ab_uni = make_filter_u(0, 0, "dehaze-ab-u");

        let apply_buf = self.uniform(
            &DehazeApplyParams {
                size: [w, h],
                lo_size: [lw, lh],
                atmosphere: [atm[0], atm[1], atm[2], 1.0],
                amount,
                scale: scale as f32,
                _pad: [0.0; 2],
            },
            "dehaze-apply-u",
        );

        let src_view = full_view(src);
        let lo_src_view = full_view(&lo_src);
        let lo_src_store_view = mip_view(&lo_src, 0);
        let dn_view = full_view(&dn);
        let dn_h_view = full_view(&dn_h);
        let dn_min_view = full_view(&dn_min);
        let packed_view = full_view(&packed);
        let packed_h_view = full_view(&packed_h);
        let packed_v_view = full_view(&packed_v);
        let ab_view = full_view(&ab);
        let ab_h_view = full_view(&ab_h);
        let ab_v_view = full_view(&ab_v);
        let out_view = full_view(&out);

        let p = &self.passes.dehaze;
        let bg_downsample = bind_group(
            device,
            "dehaze-downsample-bg",
            &p.downsample_layout,
            &[
                downsample_buf.as_entire_binding(),
                tex(&src_view),
                samp(&p.linear_sampler),
                tex(&lo_src_store_view),
            ],
        );
        let bg_norm = bind_group(
            device,
            "dehaze-norm-bg",
            &p.norm_layout,
            &[
                norm_buf.as_entire_binding(),
                tex(&lo_src_view),
                tex(&dn_view),
            ],
        );
        let bg_min_h = bind_group(
            device,
            "dehaze-min-h-bg",
            &p.min_layout,
            &[
                min_h_buf.as_entire_binding(),
                tex(&dn_view),
                tex(&dn_h_view),
            ],
        );
        let bg_min_v = bind_group(
            device,
            "dehaze-min-v-bg",
            &p.min_layout,
            &[
                min_v_buf.as_entire_binding(),
                tex(&dn_h_view),
                tex(&dn_min_view),
            ],
        );
        let bg_pack = bind_group(
            device,
            "dehaze-pack-bg",
            &p.pack_layout,
            &[
                pack_buf.as_entire_binding(),
                tex(&lo_src_view),
                tex(&dn_min_view),
                tex(&packed_view),
            ],
        );
        let bg_box_h_pack = bind_group(
            device,
            "dehaze-box-h-pack-bg",
            &p.box_layout,
            &[
                box_h_buf.as_entire_binding(),
                tex(&packed_view),
                tex(&packed_h_view),
            ],
        );
        let bg_box_v_pack = bind_group(
            device,
            "dehaze-box-v-pack-bg",
            &p.box_layout,
            &[
                box_v_buf.as_entire_binding(),
                tex(&packed_h_view),
                tex(&packed_v_view),
            ],
        );
        let bg_ab = bind_group(
            device,
            "dehaze-ab-bg",
            &p.ab_layout,
            &[
                ab_uni.as_entire_binding(),
                tex(&packed_v_view),
                tex(&ab_view),
            ],
        );
        let bg_box_h_ab = bind_group(
            device,
            "dehaze-box-h-ab-bg",
            &p.box_layout,
            &[
                box_h_buf.as_entire_binding(),
                tex(&ab_view),
                tex(&ab_h_view),
            ],
        );
        let bg_box_v_ab = bind_group(
            device,
            "dehaze-box-v-ab-bg",
            &p.box_layout,
            &[
                box_v_buf.as_entire_binding(),
                tex(&ab_h_view),
                tex(&ab_v_view),
            ],
        );
        let bg_apply = bind_group(
            device,
            "dehaze-apply-bg",
            &p.apply_layout,
            &[
                apply_buf.as_entire_binding(),
                tex(&src_view),
                tex(&ab_v_view),
                tex(&out_view),
            ],
        );

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("dehaze-enc"),
        });
        let gx_lo = lw.div_ceil(16);
        let gy_lo = lh.div_ceil(16);
        let gx = w.div_ceil(16);
        let gy = h.div_ceil(16);
        {
            let mut c = begin_pass(&mut encoder, "dehaze-pass");
            c.set_pipeline(&p.downsample_pipeline);
            c.set_bind_group(0, &bg_downsample, &[]);
            c.dispatch_workgroups(gx_lo, gy_lo, 1);
            c.set_pipeline(&p.norm_pipeline);
            c.set_bind_group(0, &bg_norm, &[]);
            c.dispatch_workgroups(gx_lo, gy_lo, 1);
            c.set_pipeline(&p.min_pipeline);
            c.set_bind_group(0, &bg_min_h, &[]);
            c.dispatch_workgroups(gx_lo, gy_lo, 1);
            c.set_bind_group(0, &bg_min_v, &[]);
            c.dispatch_workgroups(gx_lo, gy_lo, 1);
            c.set_pipeline(&p.pack_pipeline);
            c.set_bind_group(0, &bg_pack, &[]);
            c.dispatch_workgroups(gx_lo, gy_lo, 1);
            c.set_pipeline(&p.box_pipeline);
            c.set_bind_group(0, &bg_box_h_pack, &[]);
            c.dispatch_workgroups(gx_lo, gy_lo, 1);
            c.set_bind_group(0, &bg_box_v_pack, &[]);
            c.dispatch_workgroups(gx_lo, gy_lo, 1);
            c.set_pipeline(&p.ab_pipeline);
            c.set_bind_group(0, &bg_ab, &[]);
            c.dispatch_workgroups(gx_lo, gy_lo, 1);
            c.set_pipeline(&p.box_pipeline);
            c.set_bind_group(0, &bg_box_h_ab, &[]);
            c.dispatch_workgroups(gx_lo, gy_lo, 1);
            c.set_bind_group(0, &bg_box_v_ab, &[]);
            c.dispatch_workgroups(gx_lo, gy_lo, 1);
            c.set_pipeline(&p.apply_pipeline);
            c.set_bind_group(0, &bg_apply, &[]);
            c.dispatch_workgroups(gx, gy, 1);
        }
        self.ctx.queue.submit(Some(encoder.finish()));
        Ok(out)
    }
}
