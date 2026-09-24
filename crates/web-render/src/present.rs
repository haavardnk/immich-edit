use raw_pipeline::gpu::context::GpuContext;
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, CommandEncoder, CompositeAlphaMode,
    CurrentSurfaceTexture, LoadOp, Operations, PipelineLayoutDescriptor, PresentMode,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    ShaderStages, StoreOp, Surface, SurfaceColorSpace, SurfaceConfiguration, SurfaceTarget,
    SurfaceTexture, Texture, TextureFormat, TextureSampleType, TextureUsages, TextureView,
    TextureViewDescriptor, TextureViewDimension,
};

pub struct Presenter {
    surface: Surface<'static>,
    format: TextureFormat,
    layout: BindGroupLayout,
    pipeline: RenderPipeline,
    configured: Option<(u32, u32, SurfaceColorSpace)>,
}

pub struct Target {
    texture: SurfaceTexture,
    pub view: TextureView,
}

impl Target {
    pub fn present(self, ctx: &GpuContext) {
        ctx.queue.present(self.texture);
    }
}

impl Presenter {
    pub fn new(ctx: &GpuContext, canvas: web_sys::OffscreenCanvas) -> Result<Self, String> {
        let surface = ctx
            .instance
            .create_surface(SurfaceTarget::OffscreenCanvas(canvas))
            .map_err(|e| format!("canvas surface: {e}"))?;
        let format = surface
            .get_capabilities(&ctx.adapter)
            .formats
            .into_iter()
            .find(|f| matches!(f, TextureFormat::Bgra8Unorm | TextureFormat::Rgba8Unorm))
            .ok_or("the canvas offers no 8-bit unorm format")?;
        let device = &ctx.device;
        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("present-bgl"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: false },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }],
        });
        let module = device.create_shader_module(wgpu::include_wgsl!("blit.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("present-layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("present"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(format.into())],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        Ok(Self {
            surface,
            format,
            layout,
            pipeline,
            configured: None,
        })
    }

    pub fn acquire(
        &mut self,
        ctx: &GpuContext,
        (width, height): (u32, u32),
        color_space: SurfaceColorSpace,
    ) -> Result<Target, String> {
        if self.configured != Some((width, height, color_space)) {
            self.configure(ctx, width, height, color_space);
        }
        let texture = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(t) | CurrentSurfaceTexture::Suboptimal(t) => t,
            CurrentSurfaceTexture::Outdated | CurrentSurfaceTexture::Lost => {
                self.configure(ctx, width, height, color_space);
                match self.surface.get_current_texture() {
                    CurrentSurfaceTexture::Success(t) | CurrentSurfaceTexture::Suboptimal(t) => t,
                    _ => return Err("the canvas surface is unavailable".into()),
                }
            }
            _ => return Err("the canvas surface is unavailable".into()),
        };
        let view = texture
            .texture
            .create_view(&TextureViewDescriptor::default());
        Ok(Target { texture, view })
    }

    pub fn blit(
        &self,
        ctx: &GpuContext,
        encoder: &mut CommandEncoder,
        src: &Texture,
        target: &TextureView,
    ) {
        let src_view = src.create_view(&TextureViewDescriptor::default());
        let bind = ctx.device.create_bind_group(&BindGroupDescriptor {
            label: Some("present-bg"),
            layout: &self.layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&src_view),
            }],
        });
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("present-pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(wgpu::Color::BLACK),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind, &[]);
        pass.draw(0..3, 0..1);
    }

    fn configure(
        &mut self,
        ctx: &GpuContext,
        width: u32,
        height: u32,
        color_space: SurfaceColorSpace,
    ) {
        self.surface.configure(
            &ctx.device,
            &SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: self.format,
                width,
                height,
                present_mode: PresentMode::Fifo,
                desired_maximum_frame_latency: 2,
                alpha_mode: CompositeAlphaMode::Opaque,
                view_formats: Vec::new(),
                color_space,
            },
        );
        self.configured = Some((width, height, color_space));
    }
}
