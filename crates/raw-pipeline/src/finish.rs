use fast_image_resize::images::Image;
use fast_image_resize::{FilterType, PixelType, ResizeAlg, ResizeOptions, Resizer};

use crate::encode::{encode_from_rgb8, encode_from_rgb16};
use crate::frame::RenderOptions;

mod sharpen;
mod watermark;

pub use watermark::{WATERMARK_MAX_EDGE, WATERMARK_MAX_SOURCE_BYTES, decode_watermark};

pub enum FinalPixels {
    Rgb8(Vec<u8>),
    Rgb16(Vec<u16>),
}

pub struct FinalImage {
    pub pixels: FinalPixels,
    pub width: u32,
    pub height: u32,
}

pub fn scale_to_edge(w: u32, h: u32, edge: u32) -> (u32, u32) {
    let scale = f64::from(edge) / f64::from(w.max(h).max(1));
    let nw = (f64::from(w) * scale).round() as u32;
    let nh = (f64::from(h) * scale).round() as u32;
    (nw.max(1), nh.max(1))
}

fn enlarged_dims(w: u32, h: u32, opts: &RenderOptions) -> Option<(u32, u32)> {
    (opts.enlarge && w.max(h) < opts.max_edge).then(|| scale_to_edge(w, h, opts.max_edge))
}

pub fn has_final_stage(w: u32, h: u32, opts: &RenderOptions) -> bool {
    enlarged_dims(w, h, opts).is_some() || opts.output_sharpen.is_some() || opts.watermark.is_some()
}

fn resize(src: Vec<u8>, w: u32, h: u32, dims: (u32, u32), pixel: PixelType) -> Option<Vec<u8>> {
    let src_image = Image::from_vec_u8(w, h, src, pixel).ok()?;
    let mut dst_image = Image::new(dims.0, dims.1, pixel);
    Resizer::new()
        .resize(
            &src_image,
            &mut dst_image,
            Some(&ResizeOptions::new().resize_alg(ResizeAlg::Convolution(FilterType::Lanczos3))),
        )
        .ok()?;
    Some(dst_image.into_vec())
}

fn enlarge(image: FinalImage, dims: (u32, u32)) -> crate::PipelineResult<FinalImage> {
    let failed =
        || crate::PipelineError::Render(format!("enlarge to {}x{} failed", dims.0, dims.1));
    let pixels = match image.pixels {
        FinalPixels::Rgb8(rgb) => FinalPixels::Rgb8(
            resize(rgb, image.width, image.height, dims, PixelType::U8x3).ok_or_else(failed)?,
        ),
        FinalPixels::Rgb16(rgb) => {
            let bytes = bytemuck::cast_slice::<u16, u8>(&rgb).to_vec();
            let out = resize(bytes, image.width, image.height, dims, PixelType::U16x3)
                .ok_or_else(failed)?;
            FinalPixels::Rgb16(
                out.chunks_exact(2)
                    .map(|b| u16::from_ne_bytes([b[0], b[1]]))
                    .collect(),
            )
        }
    };
    Ok(FinalImage {
        pixels,
        width: dims.0,
        height: dims.1,
    })
}

pub fn final_stage(image: FinalImage, opts: &RenderOptions) -> crate::PipelineResult<FinalImage> {
    let mut image = match enlarged_dims(image.width, image.height, opts) {
        Some(dims) => enlarge(image, dims)?,
        None => image,
    };
    if let Some(output_sharpen) = opts.output_sharpen {
        sharpen::sharpen(&mut image, output_sharpen);
    }
    if let Some(mark) = &opts.watermark {
        watermark::composite(&mut image, mark, opts.output_color_space)?;
    }
    Ok(image)
}

pub fn encode(image: &FinalImage, opts: &RenderOptions) -> crate::PipelineResult<Vec<u8>> {
    match &image.pixels {
        FinalPixels::Rgb8(rgb) => encode_from_rgb8(
            rgb,
            image.width,
            image.height,
            &opts.output,
            opts.output_color_space,
        ),
        FinalPixels::Rgb16(rgb) => encode_from_rgb16(
            rgb,
            image.width,
            image.height,
            &opts.output,
            opts.output_color_space,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(max_edge: u32, enlarge: bool) -> RenderOptions {
        RenderOptions {
            max_edge,
            enlarge,
            ..Default::default()
        }
    }

    fn gradient(w: u32, h: u32) -> FinalImage {
        let rgb = (0..w * h)
            .flat_map(|i| {
                let v = ((i % w) * 255 / (w - 1)) as u8;
                [v, v, v]
            })
            .collect();
        FinalImage {
            pixels: FinalPixels::Rgb8(rgb),
            width: w,
            height: h,
        }
    }

    #[test]
    fn enlarges_to_the_long_edge_keeping_the_aspect() {
        let out = final_stage(gradient(40, 30), &opts(100, true)).unwrap();
        assert_eq!((out.width, out.height), (100, 75));
        let FinalPixels::Rgb8(rgb) = out.pixels else {
            panic!("expected rgb8");
        };
        assert_eq!(rgb.len(), 100 * 75 * 3);
        assert!(rgb[0] < 10 && rgb[99 * 3] > 245);
    }

    #[test]
    fn enlarges_sixteen_bit_output() {
        let image = FinalImage {
            pixels: FinalPixels::Rgb16(vec![1000; 20 * 10 * 3]),
            width: 20,
            height: 10,
        };
        let out = final_stage(image, &opts(40, true)).unwrap();
        let FinalPixels::Rgb16(rgb) = out.pixels else {
            panic!("expected rgb16");
        };
        assert_eq!((out.width, out.height, rgb.len()), (40, 20, 40 * 20 * 3));
        assert!(rgb.iter().all(|&v| v.abs_diff(1000) <= 1));
    }

    #[test]
    fn leaves_the_image_alone_without_enlarge_or_when_already_large() {
        for options in [opts(100, false), opts(40, true), opts(30, true)] {
            assert!(!has_final_stage(40, 30, &options));
            let out = final_stage(gradient(40, 30), &options).unwrap();
            assert_eq!((out.width, out.height), (40, 30));
        }
    }

    #[test]
    fn scale_to_edge_rounds_and_never_collapses() {
        assert_eq!(scale_to_edge(3000, 2000, 6000), (6000, 4000));
        assert_eq!(scale_to_edge(2000, 3000, 4500), (3000, 4500));
        assert_eq!(scale_to_edge(10000, 1, 100), (100, 1));
    }
}
