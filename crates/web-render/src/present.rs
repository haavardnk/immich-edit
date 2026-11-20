use raw_pipeline::gpu::context::GpuContext;
use raw_pipeline::gpu::readback::{copy_texture_to_buffer, make_readback_buffer};
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, CommandEncoder, Extent3d, LoadOp,
    Operations, PipelineLayoutDescriptor, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, ShaderStages, StoreOp, Texture, TextureDescriptor,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureViewDescriptor,
    TextureViewDimension,
};

const FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;

struct Target {
    dims: (u32, u32),
    texture: Texture,
    buffer: Buffer,
}

pub struct FrameReader {
    layout: BindGroupLayout,
    pipeline: RenderPipeline,
    target: Option<Target>,
}

impl FrameReader {
    pub fn new(ctx: &GpuContext) -> Self {
        let device = &ctx.device;
        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("frame-bgl"),
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
            label: Some("frame-layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("frame"),
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
                targets: &[Some(FORMAT.into())],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        Self {
            layout,
            pipeline,
            target: None,
        }
    }

    pub fn record(
        &mut self,
        ctx: &GpuContext,
        encoder: &mut CommandEncoder,
        src: &Texture,
        dims: (u32, u32),
    ) -> Buffer {
        self.ensure(ctx, dims);
        let target = self.target.as_ref().expect("ensure sets the frame target");
        let view = target
            .texture
            .create_view(&TextureViewDescriptor::default());
        let src_view = src.create_view(&TextureViewDescriptor::default());
        let bind = ctx.device.create_bind_group(&BindGroupDescriptor {
            label: Some("frame-bg"),
            layout: &self.layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&src_view),
            }],
        });
        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("frame-pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
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
        copy_texture_to_buffer(encoder, &target.texture, &target.buffer, dims.0, dims.1);
        target.buffer.clone()
    }

    fn ensure(&mut self, ctx: &GpuContext, dims: (u32, u32)) {
        if self.target.as_ref().is_none_or(|t| t.dims != dims) {
            let texture = ctx.device.create_texture(&TextureDescriptor {
                label: Some("frame-target"),
                size: Extent3d {
                    width: dims.0,
                    height: dims.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: FORMAT,
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let buffer = make_readback_buffer(&ctx.device, dims.0, dims.1);
            self.target = Some(Target {
                dims,
                texture,
                buffer,
            });
        }
    }
}
