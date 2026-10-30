mod context;
mod finish;
mod linear;
mod shape;

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
use crate::ops::GpuRoute;
use crate::source::LinearKind;
use crate::timing;
use crate::{PipelineError, PipelineResult};
pub(crate) use context::GlobalContext;
pub(super) use context::{GridView, HazeGrid, LevelView, Placed, PresenceLevels, ShadowLevels};
pub(super) use finish::FinishStage;
#[cfg(feature = "native")]
pub(super) use finish::final_pass_active;
#[cfg(feature = "native")]
pub(super) use linear::layer_presence;
pub(super) use shape::OutputShape;

pub(super) struct DisplayInput<'s> {
    pub pass: &'s ProcessFastPass,
    pub meta: &'s FrameMeta,
    pub edits: &'s Edits,
    pub opts: &'s RenderOptions,
    pub shape: OutputShape,
}

pub(super) struct DisplayRequest<'r> {
    pub source: &'r LinearSource,
    pub edits: &'r Edits,
    pub opts: &'r RenderOptions,
    pub shape: OutputShape,
    pub context: Option<&'r GlobalContext>,
}

#[derive(Clone, Copy)]
pub(super) struct DisplayTarget<'t> {
    pub texture: &'t Texture,
    pub depth: DisplayDepth,
    pub dims: (u32, u32),
    pub full: (u32, u32),
    pub origin: (u32, u32),
    pub scale: f32,
}

pub(super) struct StageState {
    pub texture: Arc<Texture>,
    pub extent: SourceExtent,
    pub origin: (u32, u32),
    pub shadows: Option<ShadowLevels>,
    atmosphere: Option<[f32; 3]>,
    layers: HashMap<String, Arc<Texture>>,
    held: Vec<PooledTexture>,
}

impl StageState {
    pub fn new(source: &LinearSource) -> Self {
        Self {
            texture: source.texture.clone(),
            extent: source.extent(),
            origin: source.origin(),
            shadows: None,
            atmosphere: source.atmosphere,
            layers: HashMap::new(),
            held: Vec::new(),
        }
    }

    pub fn window(&self) -> [u32; 4] {
        let (x, y) = self.origin;
        let (w, h) = self.extent.full;
        [x, y, w, h]
    }

    pub fn placed(&self) -> Placed {
        Placed {
            extent: self.extent,
            origin: self.origin,
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
    Pooled(PooledTexture),
}

impl DisplaySlot {
    fn select(candidates: [Option<PooledTexture>; 3], scratch: &mut Vec<PooledTexture>) -> Self {
        let mut candidates = candidates.into_iter().flatten();
        let slot = candidates.next().map_or(Self::Output, Self::Pooled);
        scratch.extend(candidates);
        slot
    }

    pub fn texture<'t>(&'t self, targets: &'t OutputTargets) -> &'t Texture {
        match self {
            Self::Output => &targets.texture,
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
        let request = DisplayRequest {
            source,
            edits: &edits,
            opts,
            shape: OutputShape::new(&source.meta, &edits, opts.max_edge, source.extent().full),
            context: source.context.as_deref(),
        };
        self.display_chain(request, RenderTimings::new(&self.ctx), None)
    }

    pub(super) fn display_chain<'a>(
        &'a self,
        request: DisplayRequest<'_>,
        timings: RenderTimings<'a>,
        cancel: Option<&CancelToken>,
    ) -> PipelineResult<DisplayFrame<'a>> {
        let DisplayRequest {
            source,
            edits,
            opts,
            shape,
            context,
        } = request;
        let edits_c = edits.clamped();
        let meta = &source.meta;
        let depth = display_depth(opts);
        OutputTargets::check_limits(&self.ctx.device.limits(), shape.dims(), depth)?;
        let passes = match depth {
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
                shape,
            };
            return self.encode_display(input, state, timings, cancel);
        }

        let t = &timings;
        self.dehaze_stage(&mut state, &edits_c, context, t, cancel)?;
        self.lens_blur_stage(&mut state, meta, &edits_c, opts, t, cancel)?;
        let base = state.texture.clone();
        let levels = self.presence_levels(&mut state, &edits_c, context, t, cancel)?;
        self.presence_stage(&mut state, &edits_c, &levels, t, cancel)?;
        self.layer_presence_stage(&mut state, &base, &edits_c, &levels, t, cancel)?;
        self.shadows_stage(&mut state, &edits_c, context, t)?;
        let input = DisplayInput {
            pass: passes.1,
            meta,
            edits,
            opts,
            shape,
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
            shape,
        } = input;
        let t = &timings;
        let mut edits = edits.clamped();
        edits.detail.sharpen_amount = Some(edits.detail.sharpen_amount_for(meta.is_raw));
        let edits = edits;
        self.check_fused_ops(&edits)?;
        self.resample_stage(&mut state, &shape, t)?;
        crate::cancel::check(cancel)?;

        let out_dims = shape.dims();
        let plan = ProcessPlan::new(meta, &edits, opts, &state, &shape);
        let targets = pools::acquire_target(&self.output_pool, &self.ctx, out_dims.0, out_dims.1)?;
        let p = &targets[0];
        let depth = display_depth(opts);
        let sixteen = self.display_texture(depth, out_dims);
        let display = DisplayTarget {
            texture: sixteen.as_deref().unwrap_or(&p.texture),
            depth,
            dims: out_dims,
            full: shape.full,
            origin: shape.origin(),
            scale: shape.scale,
        };
        let dcp_base = self.dcp_base_table_view(plan.ctx_op.render.dcp.as_deref());
        let views = ProcessViews::new(&state, p, display.texture, &self.dummy_texture, dcp_base);

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
            masks: &masks,
        };
        let sharpen = self.encode_finish(&mut encoder, finish, &mut scratch, &mut retained)?;
        let lut = self.maybe_encode_lut(&mut encoder, &edits, opts, display);
        let graded = lut.as_deref().unwrap_or(display.texture);
        let overlay = masks.preview_weight.map(|weight| {
            self.encode_mask_weight_image(&mut encoder, weight, out_dims, &mut retained)
        });
        retained.textures.extend(masks.sharpen);
        let display_src = overlay.as_deref().unwrap_or(graded);
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
            shape,
        };
        self.encode_meta_bins(&mut encoder, p, sources, bins, t);
        let slot = DisplaySlot::select([overlay, lut, sixteen], &mut scratch);
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
