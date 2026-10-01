use wgpu::{CommandEncoder, Texture, TextureView};

use super::GpuRenderer;
use super::display::StageState;
use super::geometry::ProcessGeom;
use super::masks::Retained;
use super::uniform::{build_process_uniform, header_uniform};
use crate::edits::Edits;
use crate::frame::{FrameMeta, RenderOptions};
use crate::gpu::dispatch::{bind_group, dispatch_2d, tex};
use crate::gpu::passes::process::ProcessFastPass;
use crate::gpu::resources::OutputTargets;
use crate::gpu::texture::full_view;
use crate::gpu::uniforms::ProcessHeader;
use crate::ops::{OpContext, OpScratch, RenderContext};

pub(super) struct ProcessPlan {
    pub geom: ProcessGeom,
    pub ctx_op: OpContext,
    pub header: ProcessHeader,
    pub layer_header: ProcessHeader,
    pub out_dims: (u32, u32),
}

impl ProcessPlan {
    pub fn new(
        meta: &FrameMeta,
        edits: &Edits,
        opts: &RenderOptions,
        state: &StageState,
        out_dims: (u32, u32),
    ) -> Self {
        let geom = ProcessGeom::new(meta, edits, state.extent.full);
        let setup = crate::dcp::setup::resolve(meta, edits, opts.dcp.as_deref());
        let ctx_op = OpContext {
            render: RenderContext {
                wb_coeffs: meta.wb_coeffs,
                cam_to_srgb: setup.cam_to_srgb,
                is_raw: meta.is_raw,
                capture_sigma: meta.capture_sigma,
                preview_mode: opts.preview_mode.clone(),
                roi: opts.roi,
                dcp: setup.resolved,
                output_scale: crate::geom::output_scale(
                    meta.orientation,
                    edits,
                    (meta.width as u32, meta.height as u32),
                    out_dims,
                ),
            },
            scratch: OpScratch::default(),
        };
        let header = |warp: bool| {
            header_uniform(
                edits,
                &geom,
                state.extent.dims,
                state.window,
                out_dims,
                state.shadows_mip,
                warp,
            )
        };
        Self {
            header: header(true),
            layer_header: header(false),
            geom,
            ctx_op,
            out_dims,
        }
    }
}

pub(super) struct ProcessViews {
    pub src: TextureView,
    pub out: TextureView,
    pub linear: TextureView,
    pub shadows: TextureView,
    pub dcp_base: TextureView,
}

impl ProcessViews {
    pub fn new(
        state: &StageState,
        target: &OutputTargets,
        display: &Texture,
        dummy_luma: &Texture,
        dcp_base: TextureView,
    ) -> Self {
        Self {
            src: full_view(&state.texture),
            out: full_view(display),
            linear: full_view(&target.linear_texture),
            shadows: full_view(state.shadows.as_deref().unwrap_or(dummy_luma)),
            dcp_base,
        }
    }
}

impl GpuRenderer {
    pub(super) fn encode_process(
        &self,
        encoder: &mut CommandEncoder,
        pass: &ProcessFastPass,
        edits: &Edits,
        plan: &ProcessPlan,
        views: &ProcessViews,
        retained: &mut Retained,
    ) {
        let device = &self.ctx.device;
        let bytes = build_process_uniform(
            &pass.built,
            &self.passes.registry,
            edits,
            &plan.ctx_op,
            &plan.header,
        );
        let uniform = self
            .uniform_pool
            .acquire(device, &self.ctx.queue, &bytes, "process-uniform");
        let bind = bind_group(
            device,
            "process-bg",
            &pass.layout,
            &[
                uniform.as_entire_binding(),
                tex(&views.src),
                tex(&views.out),
                tex(&views.linear),
                tex(&views.shadows),
                tex(&views.dcp_base),
            ],
        );
        dispatch_2d(
            encoder,
            "process-pass",
            &pass.pipeline,
            &bind,
            plan.out_dims.0.div_ceil(16),
            plan.out_dims.1.div_ceil(16),
        );
        retained.uniforms.push(uniform);
        retained.binds.push(bind);
    }
}
