use std::sync::Arc;

use wgpu::{Buffer, CommandEncoder, CommandEncoderDescriptor, Texture, TextureFormat};

use crate::gpu::dispatch::{bind_group, buf, dispatch_2d, tex};
use crate::gpu::passes::nr::{NR_HIST_TILE, NrAtrousParams, NrKernel, NrPlaneKernels};
use crate::gpu::renderer::GpuRenderer;
use crate::gpu::texture::{STORAGE_SAMPLED, mip_view};
use crate::gpu::texture_pool::{PooledTexture, TextureKey};
use crate::gpu::uniform_pool::PooledUniform;

const GROUP: u32 = 16;

pub(super) struct NrSweep<'r> {
    renderer: &'r GpuRenderer,
    encoder: CommandEncoder,
    uniforms: Vec<PooledUniform>,
    scratch: Vec<PooledTexture>,
}

impl<'r> NrSweep<'r> {
    pub fn new(renderer: &'r GpuRenderer, label: &'static str) -> Self {
        let encoder = renderer
            .ctx
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: Some(label) });
        Self {
            renderer,
            encoder,
            uniforms: Vec::new(),
            scratch: Vec::new(),
        }
    }

    pub fn scratch(
        &mut self,
        format: TextureFormat,
        (w, h): (u32, u32),
        label: &'static str,
    ) -> Arc<Texture> {
        let texture = self.renderer.texture_pool.acquire(
            &self.renderer.ctx.device,
            TextureKey::new(format, w, h, 1, STORAGE_SAMPLED),
            label,
        );
        let shared = texture.shared();
        self.scratch.push(texture);
        shared
    }

    pub fn run<T: bytemuck::Pod>(
        &mut self,
        label: &'static str,
        kernel: &NrKernel,
        params: &T,
        textures: &[&Texture],
        buffer: Option<&Buffer>,
        groups: (u32, u32),
    ) {
        let uniform = self.renderer.uniform(params, label);
        {
            let views: Vec<_> = textures.iter().map(|t| mip_view(t, 0)).collect();
            let mut resources = vec![uniform.as_entire_binding()];
            resources.extend(views.iter().map(tex));
            resources.extend(buffer.map(buf));
            let bind = bind_group(&self.renderer.ctx.device, label, &kernel.layout, &resources);
            dispatch_2d(
                &mut self.encoder,
                label,
                &kernel.pipeline,
                &bind,
                groups.0,
                groups.1,
            );
        }
        self.uniforms.push(uniform);
    }

    pub fn atrous(
        &mut self,
        kernels: &NrPlaneKernels,
        [src, tmp, dst]: [&Texture; 3],
        (w, h): (u32, u32),
        step: u32,
    ) {
        for (axis, from, to) in [(0, src, tmp), (1, tmp, dst)] {
            let params = NrAtrousParams {
                size: [w, h],
                step,
                axis,
            };
            self.run(
                "nr-atrous",
                &kernels.atrous,
                &params,
                &[from, to],
                None,
                groups((w, h)),
            );
        }
    }

    pub fn encoder(&mut self) -> &mut CommandEncoder {
        &mut self.encoder
    }

    pub fn submit(self) {
        self.renderer.ctx.queue.submit(Some(self.encoder.finish()));
    }
}

pub(super) fn groups((w, h): (u32, u32)) -> (u32, u32) {
    (w.div_ceil(GROUP), h.div_ceil(GROUP))
}

pub(super) fn hist_groups((w, h): (u32, u32)) -> (u32, u32) {
    (w.div_ceil(NR_HIST_TILE), h.div_ceil(NR_HIST_TILE))
}

pub(super) fn half((w, h): (u32, u32)) -> (u32, u32) {
    (w.div_ceil(2), h.div_ceil(2))
}
