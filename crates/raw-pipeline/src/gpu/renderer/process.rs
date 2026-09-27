use wgpu::{CommandEncoder, Texture, TextureView, TextureViewDescriptor};

use super::GpuRenderer;
use super::display::StageState;
use super::geometry::{ProcessGeom, process_geom};
use super::masks::Retained;
use super::uniform::{build_process_uniform, process_header};
use crate::edits::Edits;
use crate::frame::{FrameMeta, RenderOptions};
use crate::gpu::dispatch::{bind_group, dispatch_2d, tex};
use crate::gpu::passes::process::ProcessFastPass;
use crate::gpu::resources::OutputTargets;
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
        let geom = process_geom(meta, edits, state.extent.full);
        let setup = crate::dcp_pipeline::resolve(meta, edits, opts.dcp.as_deref());
        let ctx_op = OpContext {
            render: RenderContext {
                wb_coeffs: meta.wb_coeffs,
                cam_to_srgb: setup.cam_to_srgb,
                is_raw: meta.is_raw,
                capture_sigma: meta.capture_sigma,
                preview_mode: opts.preview_mode.clone(),
                roi: opts.roi,
                dcp: setup.resolved,
            },
            scratch: OpScratch::default(),
        };
        let header = |warp: bool| {
            process_header(
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
}

impl GpuRenderer {
    pub(super) fn process_views(
        &self,
        state: &StageState,
        target: &OutputTargets,
        display: &Texture,
    ) -> ProcessViews {
        let view = |texture: &Texture| texture.create_view(&TextureViewDescriptor::default());
        ProcessViews {
            src: view(&state.texture),
            out: view(display),
            linear: view(&target.linear_texture),
            shadows: view(state.shadows.as_deref().unwrap_or(&self.dummy_luma)),
        }
    }

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
