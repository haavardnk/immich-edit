use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BufferUsages, CommandEncoder, TextureFormat, TextureView};

use super::Retained;
use super::stage::MaskStage;
use crate::cpu::masked::{SELECTOR_EPS, SelectorTaps};
use crate::gpu::dispatch::{bind_group, dispatch_2d, tex};
use crate::gpu::passes::mask_selector::MaskSelectorParams;
use crate::gpu::renderer::uniform::build_process_uniform;
use crate::gpu::renderer::{GpuRenderer, display_depth};
use crate::gpu::texture::{STORAGE_SAMPLED, full_view};
use crate::gpu::texture_pool::{PooledTexture, TextureKey};

struct SelectorStep<'a> {
    mode: u32,
    dims: (u32, u32),
    offset: u32,
    inputs: [&'a TextureView; 3],
    outputs: [&'a TextureView; 3],
}

impl GpuRenderer {
    pub(super) fn encode_mask_selector(
        &self,
        encoder: &mut CommandEncoder,
        stage: &MaskStage<'_>,
        retained: &mut Retained,
    ) {
        let geom = &stage.plan.geom;
        let out_dims = stage.plan.out_dims;
        let grid_scale = out_dims.0 as f32 / (geom.crop.w * geom.bw);
        let long_edge = geom.oriented.0.max(geom.oriented.1) as f32 * grid_scale;
        let taps = SelectorTaps::for_long_edge(long_edge);
        let margin = if geom.geom_warps { 2 * taps.reach() } else { 0 };
        let dims = (out_dims.0 + 2 * margin, out_dims.1 + 2 * margin);
        let acquire = |format: TextureFormat, label: &'static str| {
            self.texture_pool.acquire(
                &self.ctx.device,
                TextureKey::new(format, dims.0, dims.1, 1, STORAGE_SAMPLED),
                label,
            )
        };
        let expanded =
            (margin > 0).then(|| self.encode_selector_source(encoder, stage, margin, retained));
        let source = full_view(
            expanded
                .as_ref()
                .map_or(&stage.target.linear_texture, PooledTexture::texture),
        );
        let guide = acquire(TextureFormat::Rgba32Float, "mask-selector-guide");
        let mean = acquire(TextureFormat::Rgba16Float, "mask-selector-mean");
        let spread = acquire(TextureFormat::Rgba16Float, "mask-selector-spread");
        let detail = acquire(TextureFormat::Rgba16Float, "mask-selector-detail");
        let hold =
            expanded.unwrap_or_else(|| acquire(TextureFormat::Rgba16Float, "mask-selector-hold"));
        let [guide_v, mean_v, spread_v, hold_v, detail_v] =
            [&guide, &mean, &spread, &hold, &detail].map(|t| full_view(t));
        let selector = full_view(&stage.target.mask_selector);
        let steps = [
            SelectorStep {
                mode: 0,
                dims,
                offset: 0,
                inputs: [&source, &source, &source],
                outputs: [&mean_v, &spread_v, &guide_v],
            },
            SelectorStep {
                mode: 1,
                dims,
                offset: 0,
                inputs: [&guide_v, &guide_v, &guide_v],
                outputs: [&mean_v, &spread_v, &selector],
            },
            SelectorStep {
                mode: 2,
                dims,
                offset: 0,
                inputs: [&mean_v, &spread_v, &guide_v],
                outputs: [&hold_v, &detail_v, &selector],
            },
            SelectorStep {
                mode: 3,
                dims,
                offset: 0,
                inputs: [&hold_v, &detail_v, &guide_v],
                outputs: [&mean_v, &spread_v, &selector],
            },
            SelectorStep {
                mode: 4,
                dims: out_dims,
                offset: margin,
                inputs: [&mean_v, &spread_v, &guide_v],
                outputs: [&hold_v, &detail_v, &selector],
            },
        ];
        for step in steps {
            self.encode_selector_step(encoder, step, dims, taps, retained);
        }
        retained
            .textures
            .extend([guide, mean, spread, detail, hold]);
    }

    fn encode_selector_source(
        &self,
        encoder: &mut CommandEncoder,
        stage: &MaskStage<'_>,
        margin: u32,
        retained: &mut Retained,
    ) -> PooledTexture {
        let device = &self.ctx.device;
        let (out_w, out_h) = stage.plan.out_dims;
        let dims = (out_w + 2 * margin, out_h + 2 * margin);
        let mut header = stage.plan.layer_header;
        let [x, y, w, h] = header.crop;
        let pad_x = w * margin as f32 / out_w as f32;
        let pad_y = h * margin as f32 / out_h as f32;
        header.crop = [x - pad_x, y - pad_y, w + 2.0 * pad_x, h + 2.0 * pad_y];
        header.out_size = [dims.0, dims.1];
        let key =
            |format: TextureFormat| TextureKey::new(format, dims.0, dims.1, 1, STORAGE_SAMPLED);
        let linear = self.texture_pool.acquire(
            device,
            key(TextureFormat::Rgba16Float),
            "mask-selector-linear",
        );
        let tone = self.texture_pool.acquire(
            device,
            key(display_depth(stage.opts).format()),
            "mask-selector-tone",
        );
        let bytes = build_process_uniform(
            &stage.pass.built,
            &self.passes.registry,
            stage.edits,
            &stage.plan.ctx_op,
            &header,
        );
        let uniform =
            self.uniform_pool
                .acquire(device, &self.ctx.queue, &bytes, "process-uniform-selector");
        let bind = bind_group(
            device,
            "process-bg-selector",
            &stage.pass.layout,
            &[
                uniform.as_entire_binding(),
                tex(&stage.views.src),
                tex(&full_view(&tone)),
                tex(&full_view(&linear)),
                tex(&stage.views.shadows),
                tex(&stage.views.dcp_base),
            ],
        );
        dispatch_2d(
            encoder,
            "process-selector",
            &stage.pass.pipeline,
            &bind,
            dims.0.div_ceil(16),
            dims.1.div_ceil(16),
        );
        retained.uniforms.push(uniform);
        retained.binds.push(bind);
        retained.textures.push(tone);
        linear
    }

    fn encode_selector_step(
        &self,
        encoder: &mut CommandEncoder,
        step: SelectorStep<'_>,
        src_size: (u32, u32),
        taps: SelectorTaps,
        retained: &mut Retained,
    ) {
        let device = &self.ctx.device;
        let params = MaskSelectorParams {
            size: [step.dims.0, step.dims.1],
            src_size: [src_size.0, src_size.1],
            offset: step.offset,
            mode: step.mode,
            whole: taps.whole,
            frac: taps.frac,
            eps: SELECTOR_EPS,
            _pad: [0; 3],
        };
        let params_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("mask-selector-uniform"),
            contents: bytemuck::bytes_of(&params),
            usage: BufferUsages::UNIFORM,
        });
        let [a, b, guide] = step.inputs;
        let [out_a, out_b, out_full] = step.outputs;
        let bind = bind_group(
            device,
            "mask-selector-bg",
            &self.passes.mask_selector.layout,
            &[
                params_buf.as_entire_binding(),
                tex(a),
                tex(b),
                tex(guide),
                tex(out_a),
                tex(out_b),
                tex(out_full),
            ],
        );
        dispatch_2d(
            encoder,
            "mask-selector",
            &self.passes.mask_selector.pipeline,
            &bind,
            step.dims.0.div_ceil(16),
            step.dims.1.div_ceil(16),
        );
        retained.bufs.push(params_buf);
        retained.binds.push(bind);
    }
}
