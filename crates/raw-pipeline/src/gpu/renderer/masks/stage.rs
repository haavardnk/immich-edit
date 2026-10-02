use std::collections::HashMap;
use std::sync::Arc;

use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BufferUsages, CommandEncoder, Texture, TextureView};

use super::{
    LAYER_LABELS, MaskAtlas, MaskWeightJob, PREVIEW_LABELS, Retained, atlas_view, mask_aspect,
};
use crate::cpu::masked::{build_layer_eval, effective_edits_for_layer};
use crate::edits::{Edits, MaskLayer};
use crate::frame::{PreviewMode, RenderOptions};
use crate::gpu::dispatch::{bind_group, copy_texture, dispatch_2d, tex};
use crate::gpu::passes::process::ProcessFastPass;
use crate::gpu::renderer::GpuRenderer;
use crate::gpu::renderer::process::{ProcessPlan, ProcessViews};
use crate::gpu::renderer::uniform::build_process_uniform;
use crate::gpu::resources::OutputTargets;
use crate::gpu::texture::full_view;

pub(in crate::gpu::renderer) struct MaskStage<'a> {
    pub pass: &'a ProcessFastPass,
    pub edits: &'a Edits,
    pub opts: &'a RenderOptions,
    pub plan: &'a ProcessPlan,
    pub views: &'a ProcessViews,
    pub layers: &'a HashMap<String, Arc<Texture>>,
    pub target: &'a OutputTargets,
}

#[derive(Default)]
pub(in crate::gpu::renderer) struct MaskStageOutput {
    pub preview_atlas: Option<MaskAtlas>,
    pub layer_atlas: Option<MaskAtlas>,
    pub has_masks: bool,
    pub preview_active: bool,
}

struct MaskViews {
    scratch_linear: TextureView,
    weight: TextureView,
    sharpen: TextureView,
    accum_alt: TextureView,
    linear: TextureView,
    base_linear: TextureView,
    atlas: TextureView,
}

impl MaskViews {
    fn new(p: &OutputTargets, atlas: &MaskAtlas) -> Self {
        Self {
            scratch_linear: full_view(&p.mask_scratch_linear),
            weight: full_view(&p.mask_weight),
            sharpen: full_view(&p.mask_sharpen),
            accum_alt: full_view(&p.mask_accum_alt),
            linear: full_view(&p.linear_texture),
            base_linear: full_view(&p.mask_base_linear),
            atlas: atlas_view(&atlas.texture),
        }
    }
}

struct MaskLayerJob<'a> {
    layer: &'a MaskLayer,
    slot_map: &'a HashMap<String, u32>,
    sharpen_flags: u32,
    accum_in_alt: bool,
}

impl GpuRenderer {
    pub(in crate::gpu::renderer) fn encode_mask_stage(
        &self,
        encoder: &mut CommandEncoder,
        stage: MaskStage<'_>,
        retained: &mut Retained,
    ) -> MaskStageOutput {
        let edits = stage.edits;
        let preview_layer = match &stage.opts.preview_mode {
            PreviewMode::MaskWeight { layer_id } => edits.masks.iter().find(|l| &l.id == layer_id),
            _ => None,
        };
        let effective_layers: Vec<&MaskLayer> = if preview_layer.is_some() {
            Vec::new()
        } else {
            edits.masks.iter().filter(|l| l.is_effective()).collect()
        };
        let mut out = MaskStageOutput {
            preview_atlas: preview_layer
                .map(|layer| self.encode_mask_preview(encoder, &stage, layer, retained)),
            has_masks: !effective_layers.is_empty(),
            preview_active: preview_layer.is_some(),
            ..Default::default()
        };
        if !out.has_masks {
            return out;
        }

        let p = stage.target;
        let out_dims = stage.plan.out_dims;
        copy_texture(encoder, &p.linear_texture, &p.mask_base_linear, out_dims);
        let (atlas, slot_map) =
            self.prepare_mask_atlas(effective_layers.iter().copied(), &stage.opts.rasters);
        let views = MaskViews::new(p, &atlas);
        out.layer_atlas = Some(atlas);

        let masked_sharpen = edits.masked_sharpen_active();
        let mut accum_in_alt = false;
        for (layer_index, layer) in effective_layers.iter().enumerate() {
            let sharpen_flags = match (masked_sharpen, layer_index) {
                (false, _) => 0,
                (true, 0) => 1,
                (true, _) => 2,
            };
            let job = MaskLayerJob {
                layer,
                slot_map: &slot_map,
                sharpen_flags,
                accum_in_alt,
            };
            self.encode_mask_layer(encoder, &stage, &views, job, retained);
            accum_in_alt = !accum_in_alt;
        }
        if accum_in_alt {
            copy_texture(encoder, &p.mask_accum_alt, &p.linear_texture, out_dims);
        }
        out
    }

    fn encode_mask_preview(
        &self,
        encoder: &mut CommandEncoder,
        stage: &MaskStage<'_>,
        layer: &MaskLayer,
        retained: &mut Retained,
    ) -> MaskAtlas {
        let (atlas, slot_map) =
            self.prepare_mask_atlas(std::iter::once(layer), &stage.opts.rasters);
        let atlas_view = atlas_view(&atlas.texture);
        let weight_view = full_view(&stage.target.mask_weight);
        let eval = build_layer_eval(layer, &stage.opts.rasters, mask_aspect(&stage.plan.geom));
        let job = MaskWeightJob {
            labels: &PREVIEW_LABELS,
            eval: &eval,
            slot_map: &slot_map,
            weight_view: &weight_view,
            atlas_view: &atlas_view,
            base_view: &stage.views.linear,
        };
        self.encode_mask_weight(
            encoder,
            job,
            stage.edits,
            &stage.plan.geom,
            stage.plan.out_dims,
            retained,
        );
        atlas
    }

    fn encode_mask_layer(
        &self,
        encoder: &mut CommandEncoder,
        stage: &MaskStage<'_>,
        views: &MaskViews,
        job: MaskLayerJob<'_>,
        retained: &mut Retained,
    ) {
        self.encode_layer_process(encoder, stage, views, job.layer, retained);
        let eval = build_layer_eval(
            job.layer,
            &stage.opts.rasters,
            mask_aspect(&stage.plan.geom),
        );
        let weight = MaskWeightJob {
            labels: &LAYER_LABELS,
            eval: &eval,
            slot_map: job.slot_map,
            weight_view: &views.weight,
            atlas_view: &views.atlas,
            base_view: &views.base_linear,
        };
        self.encode_mask_weight(
            encoder,
            weight,
            stage.edits,
            &stage.plan.geom,
            stage.plan.out_dims,
            retained,
        );
        self.encode_mask_blend(encoder, stage.plan.out_dims, views, &job, retained);
    }

    fn encode_layer_process(
        &self,
        encoder: &mut CommandEncoder,
        stage: &MaskStage<'_>,
        views: &MaskViews,
        layer: &MaskLayer,
        retained: &mut Retained,
    ) {
        let device = &self.ctx.device;
        let (out_w, out_h) = stage.plan.out_dims;
        let eff = effective_edits_for_layer(stage.edits, layer);
        let layer_src = stage.layers.get(&layer.id).map(|t| full_view(t));
        let bytes = build_process_uniform(
            &stage.pass.built,
            &self.passes.registry,
            &eff,
            &stage.plan.ctx_op,
            &stage.plan.layer_header,
        );
        let uniform =
            self.uniform_pool
                .acquire(device, &self.ctx.queue, &bytes, "process-uniform-layer");
        let bind = bind_group(
            device,
            "process-bg-layer",
            &stage.pass.layout,
            &[
                uniform.as_entire_binding(),
                tex(layer_src.as_ref().unwrap_or(&stage.views.src)),
                tex(&stage.views.out),
                tex(&views.scratch_linear),
                tex(&stage.views.shadows),
                tex(&stage.views.dcp_base),
            ],
        );
        dispatch_2d(
            encoder,
            "process-layer",
            &stage.pass.pipeline,
            &bind,
            out_w.div_ceil(16),
            out_h.div_ceil(16),
        );
        retained.uniforms.push(uniform);
        retained.binds.push(bind);
    }

    fn encode_mask_blend(
        &self,
        encoder: &mut CommandEncoder,
        (out_w, out_h): (u32, u32),
        views: &MaskViews,
        job: &MaskLayerJob<'_>,
        retained: &mut Retained,
    ) {
        let device = &self.ctx.device;
        let (curr_view, dst_view) = if job.accum_in_alt {
            (&views.accum_alt, &views.linear)
        } else {
            (&views.linear, &views.accum_alt)
        };
        let params = crate::gpu::passes::mask_blend::pack_params(
            out_w,
            out_h,
            job.layer.edits.sharpen.unwrap_or(0.0) as f32,
            job.sharpen_flags,
        );
        let params_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("mask-blend-uniform"),
            contents: bytemuck::bytes_of(&params),
            usage: BufferUsages::UNIFORM,
        });
        let bind = bind_group(
            device,
            "mask-blend-bg",
            &self.passes.mask_blend.layout,
            &[
                params_buf.as_entire_binding(),
                tex(curr_view),
                tex(&views.scratch_linear),
                tex(&views.weight),
                tex(dst_view),
                tex(&views.sharpen),
            ],
        );
        dispatch_2d(
            encoder,
            "mask-blend",
            &self.passes.mask_blend.pipeline,
            &bind,
            out_w.div_ceil(16),
            out_h.div_ceil(16),
        );
        retained.bufs.push(params_buf);
        retained.binds.push(bind);
    }
}
