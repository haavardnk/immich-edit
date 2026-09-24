use wgpu::{
    Extent3d, Origin3d, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};

use super::super::pools;
use super::*;
use crate::histogram::Bins;

struct Pixels {
    display: Vec<[u8; 3]>,
    linear: Vec<[f32; 3]>,
}

fn synthetic(width: u32, height: u32) -> Pixels {
    let count = (width * height) as usize;
    let display = (0..count)
        .map(|i| match i % 7 {
            0 => {
                let k = (i / 7 % 256) as u8;
                [k, k, k]
            }
            _ => [(i * 37) as u8, (i * 101 + 13) as u8, (i * 59 + 200) as u8],
        })
        .collect();
    let linear = (0..count)
        .map(|i| {
            let edge = (i % 256) as f32 / 255.0;
            let wide = (i * 7919 % 1400) as f32 / 1000.0 - 0.1;
            let f16 = |v: f32| half::f16::from_f32(v).to_f32();
            [f16(edge), f16(wide), f16(1.0 - wide)]
        })
        .collect();
    Pixels { display, linear }
}

fn upload(
    renderer: &GpuRenderer,
    texture: &Texture,
    bytes: &[u8],
    bytes_per_pixel: u32,
    (width, height): (u32, u32),
) {
    renderer.ctx.queue.write_texture(
        TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        },
        bytes,
        TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(width * bytes_per_pixel),
            rows_per_image: Some(height),
        },
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
}

fn display_16(renderer: &GpuRenderer, width: u32, height: u32) -> Texture {
    renderer.ctx.device.create_texture(&TextureDescriptor {
        label: Some("meta-test-display-16"),
        size: Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba16Uint,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

fn gpu_counts(renderer: &GpuRenderer, pixels: &Pixels, dims: (u32, u32), wide: bool) -> MetaCounts {
    let (width, height) = dims;
    let pool = pools::acquire_target(&renderer.output_pool, &renderer.ctx, width, height).unwrap();
    let p = &pool[0];
    let linear: Vec<u16> = pixels
        .linear
        .iter()
        .flat_map(|px| {
            let bits = |v: f32| half::f16::from_f32(v).to_bits();
            [bits(px[0]), bits(px[1]), bits(px[2]), bits(1.0)]
        })
        .collect();
    upload(
        renderer,
        &p.linear_texture,
        bytemuck::cast_slice(&linear),
        8,
        dims,
    );
    let wide_display = wide.then(|| display_16(renderer, width, height));
    match &wide_display {
        Some(texture) => {
            let words: Vec<u16> = pixels
                .display
                .iter()
                .enumerate()
                .flat_map(|(i, px)| {
                    let low = (i % 256) as u16;
                    [px[0], px[1], px[2], 255].map(|v| (v as u16) << 8 | low)
                })
                .collect();
            upload(renderer, texture, bytemuck::cast_slice(&words), 8, dims);
        }
        None => {
            let bytes: Vec<u8> = pixels
                .display
                .iter()
                .flat_map(|px| [px[0], px[1], px[2], 255])
                .collect();
            upload(renderer, &p.texture, &bytes, 4, dims);
        }
    }
    let display_src = wide_display.as_ref().unwrap_or(&p.texture);
    let request = MetaRequest {
        histogram: true,
        scopes: true,
    };
    let t = RenderTimings::new(&renderer.ctx);
    let mut encoder = renderer
        .ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    renderer.encode_meta_bins(
        &mut encoder,
        p,
        display_src,
        &p.linear_texture,
        request,
        dims,
        &t,
    );
    renderer.ctx.queue.submit(Some(encoder.finish()));
    match renderer.read_meta_counts(p, request, None) {
        Ok(counts) => counts,
        Err(e) => panic!("read meta counts: {e}"),
    }
}

fn cpu_histograms(pixels: &Pixels) -> (Histogram, Histogram) {
    let step = sample_step(pixels.display.len());
    let (display, linear) = pixels
        .display
        .iter()
        .zip(&pixels.linear)
        .step_by(step)
        .fold((Bins::zero(), Bins::zero()), |(mut d, mut l), (dp, lp)| {
            d.add_display(dp[0], dp[1], dp[2]);
            l.add_linear(lp[0], lp[1], lp[2]);
            (d, l)
        });
    (display.into_histogram(), linear.into_histogram())
}

#[test]
fn gpu_bins_match_the_cpu_counters() {
    let Ok(renderer) = GpuRenderer::new() else {
        eprintln!("no gpu adapter, skipping");
        return;
    };
    let cases = [
        ("small 8-bit", (257, 131), false),
        ("subsampled 8-bit", (1003, 601), false),
        ("subsampled 16-bit", (1003, 601), true),
    ];
    for (label, dims, wide) in cases {
        let pixels = synthetic(dims.0, dims.1);
        let counts = gpu_counts(&renderer, &pixels, dims, wide);
        let (gpu_display, gpu_linear) = counts.histograms();
        let (cpu_display, cpu_linear) = cpu_histograms(&pixels);
        if gpu_display.as_ref() != Some(&cpu_display) {
            panic!("{label}: display histogram differs");
        }
        if gpu_linear.as_ref() != Some(&cpu_linear) {
            panic!("{label}: linear histogram differs");
        }
        let rgb: Vec<u8> = pixels.display.iter().flatten().copied().collect();
        let cpu_scopes = ScopeGrids::from_rgb_u8(&rgb, dims.0 as usize, dims.1 as usize);
        let gpu_scopes = counts.scopes(&StageClock::default());
        if gpu_scopes.as_ref() != Some(&cpu_scopes) {
            panic!("{label}: scope grids differ");
        }
    }
}
