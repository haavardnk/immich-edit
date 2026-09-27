mod finish;
mod linear;

use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::MutexGuard;
use wgpu::{CommandEncoder, CommandEncoderDescriptor, Texture, TextureUsages};

use super::GpuRenderer;
use super::masks::{MaskAtlas, MaskStage, Retained};
use super::meta::{MetaRequest, MetaSources};
use super::process::{ProcessPlan, ProcessViews};
use super::{display_depth, pools};
use crate::cancel::CancelToken;
use crate::edits::Edits;
use crate::frame::{FrameMeta, RenderOptions};
use crate::gpu::display_depth::DisplayDepth;
use crate::gpu::passes::process::ProcessFastPass;
use crate::gpu::resources::{OutputTargets, SharpenTargets};
use crate::gpu::source::{LinearSource, SourceExtent};
use crate::gpu::texture::STORAGE_SAMPLED;
use crate::gpu::texture_pool::{PooledTexture, TextureKey};
use crate::gpu::timer::RenderTimings;
use crate::gpu::uniforms::FULL_WINDOW;
use crate::ops::GpuRoute;
use crate::ops::presence::{presence_mips, presence_radii};
use crate::source::LinearKind;
use crate::timing;
use crate::{PipelineError, PipelineResult};
use finish::FinishStage;

pub(super) struct DisplayInput<'s> {
    pub pass: &'s ProcessFastPass,
    pub meta: &'s FrameMeta,
    pub edits: &'s Edits,
    pub opts: &'s RenderOptions,
    pub out_dims: (u32, u32),
}

#[derive(Clone, Copy)]
pub(super) struct DisplayTarget<'t> {
    pub texture: &'t Texture,
    pub depth: DisplayDepth,
    pub dims: (u32, u32),
}

pub(super) struct StageState {
    pub texture: Arc<Texture>,
    pub extent: SourceExtent,
    pub window: [f32; 4],
    pub shadows_mip: f32,
    pub shadows: Option<Arc<Texture>>,
    atmosphere: Option<[f32; 3]>,
    layers: HashMap<String, Arc<Texture>>,
    held: Vec<PooledTexture>,
}

impl StageState {
    fn new(source: &LinearSource) -> Self {
        let extent = source.extent();
        let (full_w, full_h) = extent.full;
        let radii = presence_radii(full_w, full_h);
        Self {
            texture: source.texture.clone(),
            extent,
            window: source
                .window
                .map_or(FULL_WINDOW, |w| w.uv_rect(source.dims)),
            shadows_mip: presence_mips(full_w, full_h, radii).shadows as f32,
            shadows: None,
            atmosphere: source.atmosphere,
            layers: HashMap::new(),
            held: Vec::new(),
        }
    }

    fn hold(&mut self, texture: PooledTexture) -> Arc<Texture> {
        let shared = texture.shared();
        self.held.push(texture);
        shared
    }
}

pub(super) enum DisplaySlot {
    Output,
    Overlay,
    Pooled(PooledTexture),
}

impl DisplaySlot {
    fn select(
        overlay: bool,
        lut: Option<PooledTexture>,
        sixteen: Option<PooledTexture>,
        scratch: &mut Vec<PooledTexture>,
    ) -> Self {
        match (overlay, lut, sixteen) {
            (true, lut, sixteen) => {
                scratch.extend(lut);
                scratch.extend(sixteen);
                Self::Overlay
            }
            (false, Some(lut), sixteen) => {
                scratch.extend(sixteen);
                Self::Pooled(lut)
            }
            (false, None, Some(sixteen)) => Self::Pooled(sixteen),
            (false, None, None) => Self::Output,
        }
    }

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

    pub fn record(&mut self, f: impl FnOnce(&mut CommandEncoder, &Texture)) {
        f(&mut self.encoder, self.slot.texture(&self.targets[0]));
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
        cancel: Option<&CancelToken>,
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
        let mut state = StageState::new(source);
        if source.kind == LinearKind::PreWb {
            if super::presence_display(&edits_c) {
                return Err(PipelineError::Unsupported(
                    "presence edits need a white-balanced linear source".into(),
                ));
            }
            let input = DisplayInput {
                pass: passes.0,
                meta,
                edits,
                opts,
                out_dims,
            };
            return self.encode_display(input, state, timings, cancel);
        }

        let t = &timings;
        self.dehaze_stage(&mut state, &edits_c, t, cancel)?;
        let base = state.texture.clone();
        self.presence_stage(&mut state, &edits_c, t, cancel)?;
        self.layer_presence_stage(&mut state, &base, &edits_c, t, cancel)?;
        self.shadows_stage(&mut state, &edits_c, t)?;
        let input = DisplayInput {
            pass: passes.1,
            meta,
            edits,
            opts,
            out_dims,
        };
        self.encode_display(input, state, timings, cancel)
    }

    fn encode_display<'a>(
        &'a self,
        input: DisplayInput<'_>,
        mut state: StageState,
        timings: RenderTimings<'a>,
        cancel: Option<&CancelToken>,
    ) -> PipelineResult<DisplayFrame<'a>> {
        let DisplayInput {
            pass,
            meta,
            edits,
            opts,
            out_dims,
        } = input;
        let t = &timings;
        let mut edits = edits.clamped();
        edits.detail.sharpen_amount = Some(edits.detail.sharpen_amount_for(meta.is_raw));
        let edits = edits;
        self.check_fused_ops(&edits)?;
        self.resample_stage(&mut state, meta, &edits, out_dims, t)?;
        crate::cancel::check(cancel)?;

        let plan = ProcessPlan::new(meta, &edits, opts, &state, out_dims);
        let targets = pools::acquire_target(&self.output_pool, &self.ctx, out_dims.0, out_dims.1)?;
        let p = &targets[0];
        let depth = display_depth(opts);
        let sixteen = self.display_texture(depth, out_dims);
        let display = DisplayTarget {
            texture: sixteen.as_deref().unwrap_or(&p.texture),
            depth,
            dims: out_dims,
        };
        let views = ProcessViews::new(&state, p, display.texture, &self.dummy_luma);

        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("process-enc"),
            });
        let display_started = web_time::Instant::now();
        let display_scope = t.enter(timing::DISPLAY);
        let mut retained = Retained::default();
        self.encode_process(&mut encoder, pass, &edits, &plan, &views, &mut retained);
        let mask_stage = MaskStage {
            pass,
            edits: &edits,
            opts,
            plan: &plan,
            views: &views,
            layers: &state.layers,
            target: p,
        };
        let masks = self.encode_mask_stage(&mut encoder, mask_stage, &mut retained);

        let mut scratch = state.held;
        let finish = FinishStage {
            edits: &edits,
            opts,
            target: p,
            display,
            dcp: plan.ctx_op.render.dcp.as_deref(),
            has_masks: masks.has_masks,
        };
        let sharpen = self.encode_finish(&mut encoder, finish, &mut scratch)?;
        let lut = self.maybe_encode_lut(&mut encoder, &edits, opts, display);
        let graded = lut.as_deref().unwrap_or(display.texture);
        if masks.preview_active {
            self.encode_mask_overlay(&mut encoder, p, graded, out_dims, &mut retained);
        }
        let display_src = if masks.preview_active {
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
        let sources = MetaSources {
            display: display_src,
            linear: linear_src,
            dims: out_dims,
        };
        self.encode_meta_bins(&mut encoder, p, sources, bins, t);
        let slot = DisplaySlot::select(masks.preview_active, lut, sixteen, &mut scratch);
        Ok(DisplayFrame {
            encoder,
            targets,
            slot,
            bins,
            dims: out_dims,
            source_dims: plan.geom.source,
            atlases: [masks.preview_atlas, masks.layer_atlas],
            timings,
            sharpen,
            scratch,
            retained,
        })
    }

    fn check_fused_ops(&self, edits: &Edits) -> PipelineResult<()> {
        let missing = self
            .passes
            .registry
            .active(edits)
            .find(|op| op.gpu_route() == GpuRoute::Fused && op.gpu().is_none());
        match missing {
            Some(op) => Err(PipelineError::Unsupported(format!(
                "gpu pipeline missing op: {}",
                op.id()
            ))),
            None => Ok(()),
        }
    }

    fn display_texture(&self, depth: DisplayDepth, (w, h): (u32, u32)) -> Option<PooledTexture> {
        if depth != DisplayDepth::Sixteen {
            return None;
        }
        let usage = STORAGE_SAMPLED | TextureUsages::COPY_SRC | TextureUsages::COPY_DST;
        Some(self.texture_pool.acquire(
            &self.ctx.device,
            TextureKey::new(depth.format(), w, h, 1, usage),
            "display-16",
        ))
    }
}
