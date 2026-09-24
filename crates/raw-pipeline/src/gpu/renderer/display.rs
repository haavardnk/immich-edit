use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::MutexGuard;
use wgpu::{
    CommandEncoder, CommandEncoderDescriptor, Texture, TextureUsages, TextureViewDescriptor,
};

use super::GpuRenderer;
use super::geometry::process_geom;
use super::masks::{MaskAtlas, MaskStage, MaskStageOutput, Retained};
use super::meta::MetaRequest;
use super::{display_depth, pools, uniform};
use crate::edits::Edits;
use crate::frame::{FrameMeta, OutputColorSpace, PreviewMode, RenderOptions};
use crate::gpu::dispatch::{bind_group, dispatch_2d, samp, tex};
use crate::gpu::display_depth::DisplayDepth;
use crate::gpu::passes::process::ProcessFastPass;
use crate::gpu::resources::{OutputTargets, SharpenTargets};
use crate::gpu::source::LinearSource;
use crate::gpu::texture_pool::{PooledTexture, TextureKey};
use crate::gpu::timer::RenderTimings;
use crate::ops::{GpuRoute, OpContext, OpScratch, RenderContext};
use crate::presence::{presence_mips, presence_radii};
use crate::source::LinearKind;
use crate::timing;
use crate::{PipelineError, PipelineResult};

pub(super) struct DisplayInput<'s> {
    pub pass: &'s ProcessFastPass,
    pub src: &'s Texture,
    pub src_dims: (u32, u32),
    pub out_dims: (u32, u32),
    pub meta: &'s FrameMeta,
    pub edits: &'s Edits,
    pub opts: &'s RenderOptions,
    pub shadows: Option<&'s wgpu::TextureView>,
    pub layer_srcs: &'s HashMap<String, Arc<Texture>>,
}

pub(super) enum DisplaySlot {
    Output,
    Overlay,
    Pooled(PooledTexture),
}

impl DisplaySlot {
    pub fn texture<'t>(&'t self, targets: &'t OutputTargets) -> &'t Texture {
        match self {
            Self::Output => &targets.texture,
            Self::Overlay => &targets.mask_scratch_tone,
            Self::Pooled(texture) => texture,
        }
    }
}

pub struct DisplayFrame<'a> {
    pub(super) encoder: CommandEncoder,
    pub(super) targets: MutexGuard<'a, Vec<OutputTargets>>,
    pub(super) slot: DisplaySlot,
    pub(super) bins: MetaRequest,
    pub(super) dims: (u32, u32),
    pub(super) source_dims: (u32, u32),
    pub(super) atlases: [Option<MaskAtlas>; 2],
    pub(super) timings: RenderTimings<'a>,
    pub(super) sharpen: Option<MutexGuard<'a, Vec<SharpenTargets>>>,
    pub(super) scratch: Vec<PooledTexture>,
    pub(super) retained: Retained,
}

impl DisplayFrame<'_> {
    pub fn texture(&self) -> &Texture {
        self.slot.texture(&self.targets[0])
    }

    pub fn dims(&self) -> (u32, u32) {
        self.dims
    }
}

impl GpuRenderer {
    pub fn render_display<'a>(
        &'a self,
        source: &LinearSource,
        edits: &Edits,
        opts: &RenderOptions,
    ) -> PipelineResult<DisplayFrame<'a>> {
        if self.ctx.is_lost() {
            return Err(PipelineError::DeviceLost);
        }
        let edits = super::compose_edits(edits, opts);
        self.display_chain(source, &edits, opts, RenderTimings::new(&self.ctx), None)
    }

    pub(super) fn display_chain<'a>(
        &'a self,
        source: &LinearSource,
        edits: &Edits,
        opts: &RenderOptions,
        timings: RenderTimings<'a>,
        cancel: Option<&crate::cancel::CancelToken>,
    ) -> PipelineResult<DisplayFrame<'a>> {
        let edits_c = edits.clamped();
        let meta = &source.meta;
        let full_dims = (meta.width as u32, meta.height as u32);
        let out_dims =
            crate::geom::display_out_dims(meta.orientation, &edits_c, full_dims, opts.max_edge);
        let passes = match display_depth(opts) {
            DisplayDepth::Eight => (&self.passes.process_fast, &self.passes.process_post_wb),
            DisplayDepth::Sixteen => {
                let depth16 = self.passes.depth16(&self.ctx);
                (&depth16.process_fast, &depth16.process_post_wb)
            }
        };
        if source.kind == LinearKind::PreWb {
            if super::presence_display(&edits_c) {
                return Err(PipelineError::Unsupported(
                    "presence edits need a white-balanced linear source".into(),
                ));
            }
            let input = DisplayInput {
                pass: passes.0,
                src: &source.texture,
                src_dims: source.dims,
                out_dims,
                meta,
                edits,
                opts,
                shadows: None,
                layer_srcs: &HashMap::new(),
            };
            return self.encode_display(input, timings, cancel);
        }

        let t = &timings;
        let base = self.dehaze_source(source, &edits_c, t, cancel)?;
        let processed = if edits_c.basic.texture != 0.0 || edits_c.basic.clarity != 0.0 {
            let tex = t.stage(timing::PRESENCE, || {
                self.run_presence(&base, source.dims, &edits_c)
            })?;
            crate::cancel::check(cancel)?;
            tex
        } else {
            base.clone()
        };
        let layer_srcs = self.layer_presence_sources(&base, source.dims, &edits_c, t, cancel)?;
        let shadows_pyramid = if edits_c.tone.shadows != 0.0 {
            Some(t.stage(timing::SHADOWS, || {
                self.build_luma_pyramid(&processed, source.dims)
            })?)
        } else {
            None
        };
        let shadows_view = shadows_pyramid
            .as_ref()
            .map(|t| t.create_view(&TextureViewDescriptor::default()));
        let input = DisplayInput {
            pass: passes.1,
            src: &processed,
            src_dims: source.dims,
            out_dims,
            meta,
            edits,
            opts,
            shadows: shadows_view.as_ref(),
            layer_srcs: &layer_srcs,
        };
        self.encode_display(input, timings, cancel)
    }

    fn dehaze_source(
        &self,
        source: &LinearSource,
        edits: &Edits,
        t: &RenderTimings,
        cancel: Option<&crate::cancel::CancelToken>,
    ) -> PipelineResult<Arc<Texture>> {
        if edits.basic.dehaze == 0.0 {
            return Ok(source.texture.clone());
        }
        let dims = source.dims;
        let atmosphere = source.atmosphere.ok_or_else(|| {
            PipelineError::Unsupported("dehaze needs an atmosphere estimate on the source".into())
        })?;
        let tex = t.stage(timing::DEHAZE, || {
            let _span = tracing::debug_span!("gpu_dehaze", w = dims.0, h = dims.1).entered();
            self.run_dehaze(&source.texture, dims, edits, atmosphere)
        })?;
        crate::cancel::check(cancel)?;
        Ok(tex)
    }

    fn layer_presence_sources(
        &self,
        base: &Arc<Texture>,
        dims: (u32, u32),
        edits: &Edits,
        t: &RenderTimings,
        cancel: Option<&crate::cancel::CancelToken>,
    ) -> PipelineResult<HashMap<String, Arc<Texture>>> {
        let mut out = HashMap::new();
        let global = crate::presence::presence_amounts(edits);
        let mut cache: HashMap<(u32, u32), Arc<Texture>> = HashMap::new();
        for layer in edits.masks.iter().filter(|l| l.is_effective()) {
            let eff = crate::cpu::masked::effective_edits_for_layer(edits, layer);
            let amts = crate::presence::presence_amounts(&eff);
            if amts.texture == global.texture && amts.clarity == global.clarity {
                continue;
            }
            let key = (amts.texture.to_bits(), amts.clarity.to_bits());
            let tex = match cache.get(&key) {
                Some(tex) => tex.clone(),
                None if amts.texture == 0.0 && amts.clarity == 0.0 => base.clone(),
                None => {
                    let tex = t.stage(timing::PRESENCE, || self.run_presence(base, dims, &eff))?;
                    crate::cancel::check(cancel)?;
                    tex
                }
            };
            cache.insert(key, tex.clone());
            out.insert(layer.id.clone(), tex);
        }
        Ok(out)
    }

    fn encode_display<'a>(
        &'a self,
        input: DisplayInput<'_>,
        timings: RenderTimings<'a>,
        cancel: Option<&crate::cancel::CancelToken>,
    ) -> PipelineResult<DisplayFrame<'a>> {
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let DisplayInput {
            pass,
            src,
            src_dims,
            out_dims,
            meta,
            edits,
            opts,
            shadows,
            layer_srcs,
        } = input;
        let t = &timings;

        let mut edits = edits.clamped();
        edits.detail.sharpen_amount = Some(edits.detail.sharpen_amount_for(meta.is_raw));
        let edits = edits;
        let sharpen_active = edits.detail.sharpen_active();
        let masked_sharpen = edits.masked_sharpen_active();
        let effects_active = edits.effects.any_active();

        if let Some(op) = self
            .passes
            .registry
            .active(&edits)
            .find(|op| op.gpu_route() == GpuRoute::Fused && op.gpu().is_none())
        {
            return Err(PipelineError::Unsupported(format!(
                "gpu pipeline missing op: {}",
                op.id()
            )));
        }

        let (out_w, out_h) = out_dims;
        let (crop_w_px, crop_h_px) =
            crate::geom::display_crop_px(meta.orientation, &edits, src_dims);
        let ratio = (crop_w_px as f32 / out_w as f32).max(crop_h_px as f32 / out_h as f32);

        let (downscaled, downscaled_layers) = t.stage(timing::RESAMPLE, || {
            let Some(dims) = crate::geom::resample_target(src_dims, ratio) else {
                return Ok((None, HashMap::new()));
            };
            let main = self.resample_lanczos(src, src_dims, dims, "process-downscale")?;
            let layers = layer_srcs
                .iter()
                .map(|(id, tex)| {
                    self.resample_lanczos(tex, src_dims, dims, "layer-downscale")
                        .map(|t| (id.clone(), t))
                })
                .collect::<PipelineResult<HashMap<String, Arc<Texture>>>>()?;
            PipelineResult::Ok((Some((main, dims)), layers))
        })?;
        let (src, work_dims, layer_srcs) = match &downscaled {
            Some((tex, dims)) => (tex.as_ref(), *dims, &downscaled_layers),
            None => (src, src_dims, layer_srcs),
        };
        crate::cancel::check(cancel)?;

        let geom = process_geom(meta, &edits, work_dims);
        let setup = crate::dcp_pipeline::resolve(meta, &edits, opts.dcp.as_deref());
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
        let shadows_mip = {
            let radii = presence_radii(src_dims.0, src_dims.1);
            presence_mips(src_dims.0, src_dims.1, radii).shadows as f32
        };
        let uniform_bytes = uniform::build_process_uniform(
            &pass.built,
            &self.passes.registry,
            &edits,
            &ctx_op,
            &uniform::process_header(&edits, &geom, work_dims, out_dims, shadows_mip, true),
        );
        let uniform_buf =
            self.uniform_pool
                .acquire(device, queue, &uniform_bytes, "process-uniform");

        let src_view = src.create_view(&TextureViewDescriptor::default());
        let targets = pools::acquire_target(&self.output_pool, &self.ctx, out_w, out_h)?;
        let p = &targets[0];
        let depth = display_depth(opts);
        let sixteen = (depth == DisplayDepth::Sixteen).then(|| {
            self.texture_pool.acquire(
                device,
                TextureKey::new(
                    depth.format(),
                    out_w,
                    out_h,
                    1,
                    TextureUsages::STORAGE_BINDING
                        | TextureUsages::TEXTURE_BINDING
                        | TextureUsages::COPY_SRC
                        | TextureUsages::COPY_DST,
                ),
                "display-16",
            )
        });
        let display_tex: &Texture = sixteen.as_deref().unwrap_or(&p.texture);
        let out_view = display_tex.create_view(&TextureViewDescriptor::default());
        let linear_view = p
            .linear_texture
            .create_view(&TextureViewDescriptor::default());
        let dummy_view;
        let shadows_view = match shadows {
            Some(view) => view,
            None => {
                dummy_view = self
                    .dummy_luma
                    .create_view(&TextureViewDescriptor::default());
                &dummy_view
            }
        };

        let bind = bind_group(
            device,
            "process-bg",
            &pass.layout,
            &[
                uniform_buf.as_entire_binding(),
                tex(&src_view),
                samp(&self.passes.linear_sampler),
                tex(&out_view),
                tex(&linear_view),
                tex(shadows_view),
            ],
        );

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("process-enc"),
        });
        let display_started = web_time::Instant::now();
        let display_scope = t.enter(timing::DISPLAY);
        dispatch_2d(
            &mut encoder,
            "process-pass",
            &pass.pipeline,
            &bind,
            out_w.div_ceil(16),
            out_h.div_ceil(16),
        );

        let MaskStageOutput {
            mut retained,
            preview_atlas,
            layer_atlas,
            has_masks,
            preview_active,
        } = self.encode_mask_stage(
            &mut encoder,
            MaskStage {
                pass,
                edits: &edits,
                opts,
                geom: &geom,
                ctx_op: &ctx_op,
                target: p,
                src_view: &src_view,
                linear_view: &linear_view,
                shadows_view,
                layer_srcs,
                sensor_dims: work_dims,
                out_dims,
                shadows_mip,
                masked_sharpen,
            },
        );
        retained.uniforms.push(uniform_buf);
        retained.binds.push(bind);

        let sharpen_preview = matches!(
            opts.preview_mode,
            PreviewMode::SharpenMask | PreviewMode::SharpenRadius | PreviewMode::SharpenDetail
        );
        let dcp = ctx_op.render.dcp.as_deref();
        let p3_active = matches!(opts.output_color_space, OutputColorSpace::DisplayP3);
        let final_pass_active = sharpen_active
            || sharpen_preview
            || effects_active
            || has_masks
            || dcp.is_some()
            || p3_active
            || opts.gamut_warn
            || opts.clip_warn;
        let warn_flags = opts.gamut_warn as u32 | ((opts.clip_warn as u32) << 1);
        let sharpen = final_pass_active
            .then(|| pools::acquire_target(&self.sharpen_pool, &self.ctx, out_w, out_h))
            .transpose()?;
        let mut scratch: Vec<PooledTexture> = self
            .run_dcp_base_table(&mut encoder, dcp, &p.linear_texture, out_w, out_h)
            .into_iter()
            .collect();
        if let Some(s) = sharpen.as_ref().map(|guard| &guard[0]) {
            let run_sharpen = sharpen_active || masked_sharpen || sharpen_preview;
            if run_sharpen {
                self.encode_sharpen(
                    &mut encoder,
                    &edits,
                    p,
                    s,
                    out_w,
                    out_h,
                    &opts.preview_mode,
                    masked_sharpen,
                );
            }
            self.encode_effects_tone(
                &mut encoder,
                &edits,
                p,
                s,
                display_tex,
                depth,
                out_w,
                out_h,
                run_sharpen,
                opts.output_color_space,
                warn_flags,
                opts.roi,
            );
            scratch.extend(self.run_dcp_finish(
                &mut encoder,
                dcp,
                &s.post_lin,
                display_tex,
                depth,
                out_w,
                out_h,
                warn_flags | ((p3_active as u32) << 2),
            ));
        }

        let lut =
            self.maybe_encode_lut(&mut encoder, &edits, opts, display_tex, depth, out_w, out_h);
        let graded: &Texture = lut.as_deref().unwrap_or(display_tex);
        if preview_active {
            self.encode_mask_overlay(&mut encoder, p, graded, out_dims, &mut retained);
        }
        let display_src = if preview_active {
            &p.mask_scratch_tone
        } else {
            graded
        };
        let linear_src = match sharpen.as_ref() {
            Some(guard) => &guard[0].post_lin,
            None => &p.linear_texture,
        };
        drop(display_scope);
        t.clock()
            .add_wall(timing::DISPLAY, display_started.elapsed());
        let bins = MetaRequest {
            histogram: opts.histogram,
            scopes: opts.scopes,
        };
        self.encode_meta_bins(&mut encoder, p, display_src, linear_src, bins, out_dims, t);

        let slot = match (preview_active, lut, sixteen) {
            (true, lut, sixteen) => {
                scratch.extend(lut);
                scratch.extend(sixteen);
                DisplaySlot::Overlay
            }
            (false, Some(lut), sixteen) => {
                scratch.extend(sixteen);
                DisplaySlot::Pooled(lut)
            }
            (false, None, Some(sixteen)) => DisplaySlot::Pooled(sixteen),
            (false, None, None) => DisplaySlot::Output,
        };
        Ok(DisplayFrame {
            encoder,
            targets,
            slot,
            bins,
            dims: out_dims,
            source_dims: geom.source,
            atlases: [preview_atlas, layer_atlas],
            timings,
            sharpen,
            scratch,
            retained,
        })
    }
}
