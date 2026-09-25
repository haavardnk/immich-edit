use std::collections::HashMap;

use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BufferUsages, CommandEncoder, Extent3d, TextureView, TextureViewDescriptor};

use super::ATLAS_POOL_ITEMS;
use super::GpuRenderer;
use super::geometry::ProcessGeom;
use crate::cpu::masked::LayerEval;
use crate::edits::{Edits, MaskComponentKind, MaskLayer};
use crate::gpu::dispatch::{bind_group, buf, dispatch_2d, samp, tex};
use crate::gpu::passes::mask_weight::{
    ATLAS_DIM, ATLAS_LAYERS, COMPONENT_BYTES, MaskWeightParams, pack_layer_eval,
    resample_raster_to_atlas,
};
use crate::mask_raster::RasterMap;

pub(super) struct MaskWeightLabels {
    uniform: &'static str,
    comps: &'static str,
    poly: &'static str,
    bind: &'static str,
    dispatch: &'static str,
}

pub(super) const PREVIEW_LABELS: MaskWeightLabels = MaskWeightLabels {
    uniform: "mask-preview-uniform",
    comps: "mask-preview-comps",
    poly: "mask-preview-poly",
    bind: "mask-preview-bg",
    dispatch: "mask-preview-weight",
};

pub(super) const LAYER_LABELS: MaskWeightLabels = MaskWeightLabels {
    uniform: "mask-weight-uniform",
    comps: "mask-weight-comps",
    poly: "mask-weight-poly",
    bind: "mask-weight-bg",
    dispatch: "mask-weight",
};

#[derive(Default)]
pub(super) struct Retained {
    pub bufs: Vec<wgpu::Buffer>,
    pub uniforms: Vec<crate::gpu::uniform_pool::PooledUniform>,
    pub binds: Vec<wgpu::BindGroup>,
}

pub(super) struct MaskWeightJob<'a> {
    pub labels: &'a MaskWeightLabels,
    pub eval: &'a LayerEval,
    pub slot_map: &'a HashMap<String, u32>,
    pub weight_view: &'a TextureView,
    pub atlas_view: &'a TextureView,
    pub base_view: &'a TextureView,
}

pub(super) struct MaskAtlas {
    pub texture: wgpu::Texture,
    resident: Vec<Option<String>>,
    last_used: Vec<u64>,
    clock: u64,
}

impl MaskAtlas {
    fn new(device: &wgpu::Device) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("mask-raster-atlas"),
            size: Extent3d {
                width: ATLAS_DIM,
                height: ATLAS_DIM,
                depth_or_array_layers: ATLAS_LAYERS,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        Self {
            texture,
            resident: vec![None; ATLAS_LAYERS as usize],
            last_used: vec![0; ATLAS_LAYERS as usize],
            clock: 0,
        }
    }

    fn assign(&mut self, ids: &[&String]) -> (HashMap<String, u32>, Vec<(String, u32)>) {
        self.clock += 1;
        let mut slot_map = HashMap::new();
        let mut missing: Vec<&String> = Vec::new();
        for id in ids {
            if slot_map.contains_key(*id) {
                continue;
            }
            match self.resident.iter().position(|r| r.as_ref() == Some(*id)) {
                Some(slot) => {
                    self.last_used[slot] = self.clock;
                    slot_map.insert((*id).clone(), slot as u32);
                }
                None => missing.push(id),
            }
        }
        let mut uploads = Vec::new();
        for id in missing {
            let free = (0..self.resident.len())
                .filter(|&i| self.last_used[i] != self.clock)
                .min_by_key(|&i| (self.resident[i].is_some(), self.last_used[i]));
            let Some(slot) = free else {
                break;
            };
            self.resident[slot] = Some((*id).clone());
            self.last_used[slot] = self.clock;
            slot_map.insert((*id).clone(), slot as u32);
            uploads.push(((*id).clone(), slot as u32));
        }
        (slot_map, uploads)
    }
}

pub(super) fn atlas_view(atlas: &wgpu::Texture) -> TextureView {
    atlas.create_view(&TextureViewDescriptor {
        label: Some("mask-raster-atlas-view"),
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        ..Default::default()
    })
}

fn mask_aspect(geom: &ProcessGeom) -> f32 {
    geom.display.0 as f32 / geom.display.1.max(1) as f32
}

fn mask_weight_params(
    edits: &Edits,
    geom: &ProcessGeom,
    out_dims: (u32, u32),
    eval: &LayerEval,
    n_components: u32,
) -> MaskWeightParams {
    let (display_w, display_h) = geom.display;
    let lens =
        crate::ops::lens_distortion::LensWarpParams::from_edits(&edits.lens, display_w, display_h);
    MaskWeightParams {
        out_size: [out_dims.0, out_dims.1],
        n_components,
        layer_amount: eval.amount,
        crop: [geom.crop.x, geom.crop.y, geom.crop.w, geom.crop.h],
        flags: [
            edits.geometry.rotate as u32,
            edits.geometry.flip_h as u32,
            edits.geometry.flip_v as u32,
            eval.invert as u32,
        ],
        geom_extra2: [geom.cos_a, geom.sin_a, geom.bw, geom.bh],
        geom_extra3: [
            geom.oriented.0 as f32,
            geom.oriented.1 as f32,
            display_w as f32,
            display_h as f32,
        ],
        lens: [lens.k1, lens.k2, lens.k3, lens.zoom],
        perspective: geom.persp_rows,
    }
}

impl GpuRenderer {
    pub(super) fn encode_mask_weight(
        &self,
        encoder: &mut CommandEncoder,
        job: MaskWeightJob<'_>,
        edits: &Edits,
        geom: &ProcessGeom,
        out_dims: (u32, u32),
        retained: &mut Retained,
    ) {
        let device = &self.ctx.device;
        let (comp_bytes, n_components, poly_bytes) = pack_layer_eval(job.eval, job.slot_map);
        let params = mask_weight_params(edits, geom, out_dims, job.eval, n_components);
        let params_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some(job.labels.uniform),
            contents: bytemuck::bytes_of(&params),
            usage: BufferUsages::UNIFORM,
        });
        let comp_buf_bytes = if comp_bytes.is_empty() {
            vec![0u8; COMPONENT_BYTES]
        } else {
            comp_bytes
        };
        let comp_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some(job.labels.comps),
            contents: &comp_buf_bytes,
            usage: BufferUsages::STORAGE,
        });
        let poly_buf_bytes = if poly_bytes.is_empty() {
            vec![0u8; 8]
        } else {
            poly_bytes
        };
        let poly_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some(job.labels.poly),
            contents: &poly_buf_bytes,
            usage: BufferUsages::STORAGE,
        });
        let bind = bind_group(
            device,
            job.labels.bind,
            &self.passes.mask_weight.layout,
            &[
                params_buf.as_entire_binding(),
                buf(&comp_buf),
                tex(job.weight_view),
                tex(job.atlas_view),
                samp(&self.passes.atlas_sampler),
                tex(job.base_view),
                buf(&poly_buf),
            ],
        );
        dispatch_2d(
            encoder,
            job.labels.dispatch,
            &self.passes.mask_weight.pipeline,
            &bind,
            out_dims.0.div_ceil(16),
            out_dims.1.div_ceil(16),
        );
        retained.bufs.push(params_buf);
        retained.bufs.push(comp_buf);
        retained.bufs.push(poly_buf);
        retained.binds.push(bind);
    }

    pub(super) fn prepare_mask_atlas<'a>(
        &self,
        layers: impl Iterator<Item = &'a MaskLayer>,
        rasters: &RasterMap,
    ) -> (MaskAtlas, HashMap<String, u32>) {
        let ids: Vec<&String> = layers
            .flat_map(|layer| layer.components.iter())
            .filter(|comp| comp.enabled)
            .filter_map(|comp| match &comp.kind {
                MaskComponentKind::Brush { raster_id } => Some(raster_id),
                _ => None,
            })
            .filter(|raster_id| rasters.contains_key(*raster_id))
            .collect();
        let mut atlas = match self.atlas_pool.lock().pop() {
            Some(atlas) => atlas,
            None => {
                self.atlas_allocs
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                MaskAtlas::new(&self.ctx.device)
            }
        };
        let (slot_map, uploads) = atlas.assign(&ids);
        for (raster_id, slot) in uploads {
            let Some(raster) = rasters.get(&raster_id) else {
                continue;
            };
            let bytes = {
                let mut cache = self.atlas_cache.lock();
                if let Some(b) = cache.get(&raster_id).cloned() {
                    b
                } else {
                    let b = std::sync::Arc::new(resample_raster_to_atlas(raster));
                    cache.put(raster_id.clone(), b.clone());
                    b
                }
            };
            self.atlas_uploads
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            self.ctx.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &atlas.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: slot,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                bytes.as_slice(),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(ATLAS_DIM),
                    rows_per_image: Some(ATLAS_DIM),
                },
                Extent3d {
                    width: ATLAS_DIM,
                    height: ATLAS_DIM,
                    depth_or_array_layers: 1,
                },
            );
        }
        (atlas, slot_map)
    }

    pub(super) fn release_mask_atlas(&self, atlas: Option<MaskAtlas>) {
        let Some(atlas) = atlas else {
            return;
        };
        let mut pool = self.atlas_pool.lock();
        if pool.len() < ATLAS_POOL_ITEMS {
            pool.push(atlas);
        }
    }
}

pub(super) struct MaskStage<'a> {
    pub pass: &'a crate::gpu::passes::process::ProcessFastPass,
    pub edits: &'a Edits,
    pub opts: &'a crate::frame::RenderOptions,
    pub geom: &'a ProcessGeom,
    pub ctx_op: &'a crate::ops::OpContext,
    pub target: &'a crate::gpu::resources::OutputTargets,
    pub src_view: &'a TextureView,
    pub linear_view: &'a TextureView,
    pub shadows_view: &'a TextureView,
    pub layer_srcs: &'a HashMap<String, std::sync::Arc<wgpu::Texture>>,
    pub sensor_dims: (u32, u32),
    pub src_window: [f32; 4],
    pub out_dims: (u32, u32),
    pub shadows_mip: f32,
    pub masked_sharpen: bool,
}

#[derive(Default)]
pub(super) struct MaskStageOutput {
    pub retained: Retained,
    pub preview_atlas: Option<MaskAtlas>,
    pub layer_atlas: Option<MaskAtlas>,
    pub has_masks: bool,
    pub preview_active: bool,
}

impl GpuRenderer {
    pub(super) fn encode_mask_stage(
        &self,
        encoder: &mut CommandEncoder,
        stage: MaskStage<'_>,
    ) -> MaskStageOutput {
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let (out_w, out_h) = stage.out_dims;
        let p = stage.target;
        let edits = stage.edits;

        let preview_layer = match &stage.opts.preview_mode {
            crate::frame::PreviewMode::MaskWeight { layer_id } => {
                edits.masks.iter().find(|l| &l.id == layer_id)
            }
            _ => None,
        };
        let effective_layers: Vec<&MaskLayer> = if preview_layer.is_some() {
            Vec::new()
        } else {
            edits.masks.iter().filter(|l| l.is_effective()).collect()
        };
        let mut out = MaskStageOutput {
            has_masks: !effective_layers.is_empty(),
            preview_active: preview_layer.is_some(),
            ..Default::default()
        };

        if let Some(layer) = preview_layer {
            let (atlas, slot_map) =
                self.prepare_mask_atlas(std::iter::once(layer), &stage.opts.rasters);
            let atlas_view = atlas_view(&atlas.texture);
            let weight_view = p.mask_weight.create_view(&TextureViewDescriptor::default());
            let eval = crate::cpu::masked::build_layer_eval(
                layer,
                &stage.opts.rasters,
                mask_aspect(stage.geom),
            );
            self.encode_mask_weight(
                encoder,
                MaskWeightJob {
                    labels: &PREVIEW_LABELS,
                    eval: &eval,
                    slot_map: &slot_map,
                    weight_view: &weight_view,
                    atlas_view: &atlas_view,
                    base_view: stage.linear_view,
                },
                edits,
                stage.geom,
                stage.out_dims,
                &mut out.retained,
            );
            out.preview_atlas = Some(atlas);
        }
        if !out.has_masks {
            return out;
        }

        let scratch_linear_view = p
            .mask_scratch_linear
            .create_view(&TextureViewDescriptor::default());
        let scratch_tone_view = p
            .mask_scratch_tone
            .create_view(&TextureViewDescriptor::default());
        let weight_view = p.mask_weight.create_view(&TextureViewDescriptor::default());
        let sharpen_accum_view = p
            .mask_sharpen
            .create_view(&TextureViewDescriptor::default());
        let accum_alt_view = p
            .mask_accum_alt
            .create_view(&TextureViewDescriptor::default());
        let linear_view2 = p
            .linear_texture
            .create_view(&TextureViewDescriptor::default());
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &p.linear_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &p.mask_base_linear,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            Extent3d {
                width: out_w,
                height: out_h,
                depth_or_array_layers: 1,
            },
        );
        let base_linear_view = p
            .mask_base_linear
            .create_view(&TextureViewDescriptor::default());

        let (atlas, slot_map) =
            self.prepare_mask_atlas(effective_layers.iter().copied(), &stage.opts.rasters);
        let atlas_view = atlas_view(&atlas.texture);
        out.layer_atlas = Some(atlas);

        let mut accum_in_alt = false;
        for (layer_index, layer) in effective_layers.iter().enumerate() {
            let eff = crate::cpu::masked::effective_edits_for_layer(edits, layer);
            let layer_src_view = stage
                .layer_srcs
                .get(&layer.id)
                .map(|t| t.create_view(&TextureViewDescriptor::default()));
            let layer_src_view_ref = layer_src_view.as_ref().unwrap_or(stage.src_view);
            let eff_uniform = super::uniform::build_process_uniform(
                &stage.pass.built,
                &self.passes.registry,
                &eff,
                stage.ctx_op,
                &super::uniform::process_header(
                    edits,
                    stage.geom,
                    stage.sensor_dims,
                    stage.src_window,
                    stage.out_dims,
                    stage.shadows_mip,
                    false,
                ),
            );
            let eff_uniform_buf =
                self.uniform_pool
                    .acquire(device, queue, &eff_uniform, "process-uniform-layer");
            let layer_bind = bind_group(
                device,
                "process-bg-layer",
                &stage.pass.layout,
                &[
                    eff_uniform_buf.as_entire_binding(),
                    tex(layer_src_view_ref),
                    tex(&scratch_tone_view),
                    tex(&scratch_linear_view),
                    tex(stage.shadows_view),
                ],
            );
            dispatch_2d(
                encoder,
                "process-layer",
                &stage.pass.pipeline,
                &layer_bind,
                out_w.div_ceil(16),
                out_h.div_ceil(16),
            );
            out.retained.uniforms.push(eff_uniform_buf);
            out.retained.binds.push(layer_bind);

            let eval = crate::cpu::masked::build_layer_eval(
                layer,
                &stage.opts.rasters,
                mask_aspect(stage.geom),
            );
            self.encode_mask_weight(
                encoder,
                MaskWeightJob {
                    labels: &LAYER_LABELS,
                    eval: &eval,
                    slot_map: &slot_map,
                    weight_view: &weight_view,
                    atlas_view: &atlas_view,
                    base_view: &base_linear_view,
                },
                edits,
                stage.geom,
                stage.out_dims,
                &mut out.retained,
            );

            let (curr_view, dst_view) = if accum_in_alt {
                (&accum_alt_view, &linear_view2)
            } else {
                (&linear_view2, &accum_alt_view)
            };
            let sharpen_flags = if !stage.masked_sharpen {
                0u32
            } else if layer_index == 0 {
                1u32
            } else {
                2u32
            };
            let bl_params = crate::gpu::passes::mask_blend::pack_params(
                out_w,
                out_h,
                layer.edits.sharpen.unwrap_or(0.0) as f32,
                sharpen_flags,
            );
            let bl_params_buf = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("mask-blend-uniform"),
                contents: bytemuck::bytes_of(&bl_params),
                usage: BufferUsages::UNIFORM,
            });
            let bl_bind = bind_group(
                device,
                "mask-blend-bg",
                &self.passes.mask_blend.layout,
                &[
                    bl_params_buf.as_entire_binding(),
                    tex(curr_view),
                    tex(&scratch_linear_view),
                    tex(&weight_view),
                    tex(dst_view),
                    tex(&sharpen_accum_view),
                ],
            );
            dispatch_2d(
                encoder,
                "mask-blend",
                &self.passes.mask_blend.pipeline,
                &bl_bind,
                out_w.div_ceil(16),
                out_h.div_ceil(16),
            );
            out.retained.bufs.push(bl_params_buf);
            out.retained.binds.push(bl_bind);

            accum_in_alt = !accum_in_alt;
        }
        if accum_in_alt {
            encoder.copy_texture_to_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &p.mask_accum_alt,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyTextureInfo {
                    texture: &p.linear_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                Extent3d {
                    width: out_w,
                    height: out_h,
                    depth_or_array_layers: 1,
                },
            );
        }
        out
    }
}
