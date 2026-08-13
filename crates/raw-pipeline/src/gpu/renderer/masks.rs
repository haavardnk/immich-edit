use std::collections::HashMap;

use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BufferUsages, CommandEncoder, Extent3d, TextureView, TextureViewDescriptor};

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

pub(super) fn atlas_slot_map<'a>(
    layers: impl Iterator<Item = &'a MaskLayer>,
    rasters: &RasterMap,
) -> HashMap<String, u32> {
    let mut slot_map: HashMap<String, u32> = HashMap::new();
    let brush_rasters = layers
        .flat_map(|layer| layer.components.iter())
        .filter(|comp| comp.enabled)
        .filter_map(|comp| match &comp.kind {
            MaskComponentKind::Brush { raster_id } => Some(raster_id),
            _ => None,
        });
    for raster_id in brush_rasters {
        if slot_map.len() as u32 >= ATLAS_LAYERS {
            break;
        }
        if !slot_map.contains_key(raster_id) && rasters.contains_key(raster_id) {
            let slot = slot_map.len() as u32;
            slot_map.insert(raster_id.clone(), slot);
        }
    }
    slot_map
}

pub(super) fn atlas_view(atlas: &wgpu::Texture) -> TextureView {
    atlas.create_view(&TextureViewDescriptor {
        label: Some("mask-raster-atlas-view"),
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        ..Default::default()
    })
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

    pub(super) fn upload_mask_atlas(
        &self,
        slot_map: &HashMap<String, u32>,
        rasters: &RasterMap,
    ) -> wgpu::Texture {
        let atlas = self.ctx.device.create_texture(&wgpu::TextureDescriptor {
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
        for (raster_id, slot) in slot_map {
            let Some(raster) = rasters.get(raster_id) else {
                continue;
            };
            let bytes = {
                let mut cache = self.atlas_cache.lock();
                if let Some(b) = cache.get(raster_id).cloned() {
                    b
                } else {
                    let b = std::sync::Arc::new(resample_raster_to_atlas(raster));
                    cache.put(raster_id.clone(), b.clone());
                    b
                }
            };
            self.ctx.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &atlas,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: *slot,
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
        atlas
    }
}
