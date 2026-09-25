use fast_image_resize::PixelType;
use rayon::prelude::*;

use super::{FinalImage, FinalPixels, resize, scale_to_edge};
use crate::PipelineError;
use crate::color::srgb_lin_to_display_p3;
use crate::frame::{Align, OutputColorSpace, Watermark, WatermarkImage};
use crate::math::srgb_to_linear;
use crate::tone::srgb_oetf_scalar;

pub const WATERMARK_MAX_SOURCE_BYTES: usize = 16 * 1024 * 1024;
pub const WATERMARK_MAX_EDGE: u32 = 4096;

pub fn decode_watermark(bytes: &[u8]) -> crate::PipelineResult<WatermarkImage> {
    if bytes.len() > WATERMARK_MAX_SOURCE_BYTES {
        return Err(PipelineError::Unsupported(
            "watermark PNG is too large".into(),
        ));
    }
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(
        png::Transformations::normalize_to_color8() | png::Transformations::ALPHA,
    );
    let mut reader = decoder
        .read_info()
        .map_err(|e| PipelineError::Decode(format!("png: {e}")))?;
    let (width, height) = (reader.info().width, reader.info().height);
    if width.max(height) > WATERMARK_MAX_EDGE {
        return Err(PipelineError::Unsupported(format!(
            "watermark is {width}x{height}; the longest side may be at most {WATERMARK_MAX_EDGE} px"
        )));
    }
    let size = reader
        .output_buffer_size()
        .ok_or_else(|| PipelineError::Decode("png: image too large".into()))?;
    let mut buf = vec![0u8; size];
    let frame = reader
        .next_frame(&mut buf)
        .map_err(|e| PipelineError::Decode(format!("png: {e}")))?;
    let data = &buf[..frame.buffer_size()];
    let rgba = match frame.color_type {
        png::ColorType::Rgba => data.to_vec(),
        png::ColorType::Rgb => data
            .chunks_exact(3)
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect(),
        png::ColorType::GrayscaleAlpha => data
            .chunks_exact(2)
            .flat_map(|p| [p[0], p[0], p[0], p[1]])
            .collect(),
        png::ColorType::Grayscale => data.iter().flat_map(|&g| [g, g, g, 255]).collect(),
        png::ColorType::Indexed => {
            return Err(PipelineError::Unsupported("png: unexpanded palette".into()));
        }
    };
    Ok(WatermarkImage {
        width,
        height,
        rgba,
    })
}

struct Overlay {
    x: usize,
    y: usize,
    width: usize,
    pixels: Vec<[f32; 4]>,
}

fn offset(align: Align, outer: u32, inner: u32, inset: u32) -> u32 {
    let room = outer.saturating_sub(inner);
    match align {
        Align::Start => inset.min(room),
        Align::Center => room / 2,
        Align::End => room.saturating_sub(inset),
    }
}

fn overlay(
    watermark: &Watermark,
    width: u32,
    height: u32,
    space: OutputColorSpace,
) -> crate::PipelineResult<Overlay> {
    let short = width.min(height) as f32;
    let edge = ((short * watermark.size).round() as u32).clamp(1, width.min(height));
    let source = &watermark.image;
    let dims = scale_to_edge(source.width, source.height, edge);
    let rgba = if dims == (source.width, source.height) {
        source.rgba.clone()
    } else {
        resize(
            source.rgba.clone(),
            source.width,
            source.height,
            dims,
            PixelType::U8x4,
        )
        .ok_or_else(|| {
            PipelineError::Render(format!("watermark scale to {}x{} failed", dims.0, dims.1))
        })?
    };
    let inset = (short * watermark.inset).round() as u32;
    let pixels = rgba
        .par_chunks_exact(4)
        .map(|p| {
            let srgb = [p[0], p[1], p[2]].map(|c| f32::from(c) / 255.0);
            let rgb = match space {
                OutputColorSpace::SRgb => srgb,
                OutputColorSpace::DisplayP3 => srgb_lin_to_display_p3(srgb.map(srgb_to_linear))
                    .map(|c| srgb_oetf_scalar(c.clamp(0.0, 1.0))),
            };
            [
                rgb[0],
                rgb[1],
                rgb[2],
                f32::from(p[3]) / 255.0 * watermark.opacity,
            ]
        })
        .collect();
    Ok(Overlay {
        x: offset(watermark.anchor.x, width, dims.0, inset) as usize,
        y: offset(watermark.anchor.y, height, dims.1, inset) as usize,
        width: dims.0 as usize,
        pixels,
    })
}

fn blend<T>(rgb: &mut [T], width: usize, overlay: &Overlay, max: f32, from: fn(f32) -> T)
where
    T: Copy + Send + Sync + Into<f32>,
{
    let span = overlay.x * 3..(overlay.x + overlay.width) * 3;
    rgb.par_chunks_mut(width * 3)
        .skip(overlay.y)
        .zip(overlay.pixels.par_chunks(overlay.width))
        .for_each(|(row, src)| {
            row[span.clone()]
                .chunks_exact_mut(3)
                .zip(src)
                .filter(|(_, s)| s[3] > 0.0)
                .for_each(|(px, s)| {
                    px.iter_mut().zip(&s[..3]).for_each(|(d, &c)| {
                        let v: f32 = (*d).into();
                        *d = from((v + (c * max - v) * s[3]).clamp(0.0, max).round());
                    });
                });
        });
}

pub(super) fn composite(
    image: &mut FinalImage,
    watermark: &Watermark,
    space: OutputColorSpace,
) -> crate::PipelineResult<()> {
    let overlay = overlay(watermark, image.width, image.height, space)?;
    let width = image.width as usize;
    match &mut image.pixels {
        FinalPixels::Rgb8(rgb) => blend(rgb, width, &overlay, 255.0, |v| v as u8),
        FinalPixels::Rgb16(rgb) => blend(rgb, width, &overlay, 65535.0, |v| v as u16),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::frame::WatermarkAnchor;

    fn encode_png(width: u32, height: u32, color: png::ColorType, data: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut encoder = png::Encoder::new(&mut out, width, height);
        encoder.set_color(color);
        encoder.set_depth(png::BitDepth::Eight);
        if color == png::ColorType::Indexed {
            encoder.set_palette(vec![255, 0, 0, 0, 0, 255]);
            encoder.set_trns(vec![255, 0]);
        }
        encoder
            .write_header()
            .unwrap()
            .write_image_data(data)
            .unwrap();
        out
    }

    fn mark(width: u32, height: u32, rgba: [u8; 4]) -> Arc<WatermarkImage> {
        Arc::new(WatermarkImage {
            width,
            height,
            rgba: rgba.repeat((width * height) as usize),
        })
    }

    fn watermark(image: Arc<WatermarkImage>, anchor: (Align, Align), opacity: f32) -> Watermark {
        Watermark {
            image,
            size: 0.2,
            opacity,
            anchor: WatermarkAnchor {
                x: anchor.0,
                y: anchor.1,
            },
            inset: 0.1,
        }
    }

    fn black(width: u32, height: u32) -> FinalImage {
        FinalImage {
            pixels: FinalPixels::Rgb8(vec![0; (width * height * 3) as usize]),
            width,
            height,
        }
    }

    fn rgb8(image: &FinalImage) -> &[u8] {
        let FinalPixels::Rgb8(rgb) = &image.pixels else {
            panic!("expected rgb8");
        };
        rgb
    }

    #[test]
    fn decodes_every_png_colour_type_to_rgba() {
        let cases = [
            (
                png::ColorType::Grayscale,
                vec![7, 9],
                vec![7, 7, 7, 255, 9, 9, 9, 255],
            ),
            (
                png::ColorType::GrayscaleAlpha,
                vec![7, 1, 9, 2],
                vec![7, 7, 7, 1, 9, 9, 9, 2],
            ),
            (
                png::ColorType::Rgb,
                vec![1, 2, 3, 4, 5, 6],
                vec![1, 2, 3, 255, 4, 5, 6, 255],
            ),
            (
                png::ColorType::Indexed,
                vec![0, 1],
                vec![255, 0, 0, 255, 0, 0, 255, 0],
            ),
        ];
        for (color, data, rgba) in cases {
            let decoded = decode_watermark(&encode_png(2, 1, color, &data)).unwrap();
            assert_eq!((decoded.width, decoded.height), (2, 1), "{color:?}");
            assert_eq!(decoded.rgba, rgba, "{color:?}");
        }
    }

    #[test]
    fn rejects_oversized_and_non_png_sources() {
        let wide = encode_png(
            WATERMARK_MAX_EDGE + 1,
            1,
            png::ColorType::Grayscale,
            &[0; WATERMARK_MAX_EDGE as usize + 1],
        );
        assert!(decode_watermark(&wide).is_err());
        assert!(decode_watermark(b"not a png").is_err());
    }

    #[test]
    fn anchors_place_the_mark_inside_the_inset() {
        let cases = [
            ((Align::Start, Align::Start), (5, 5)),
            ((Align::Center, Align::Center), (45, 20)),
            ((Align::End, Align::End), (85, 35)),
            ((Align::End, Align::Start), (85, 5)),
        ];
        for (anchor, (x, y)) in cases {
            let mut image = black(100, 50);
            let mark = watermark(mark(4, 4, [255; 4]), anchor, 1.0);
            composite(&mut image, &mark, OutputColorSpace::SRgb).unwrap();
            let rgb = rgb8(&image);
            let lit: Vec<(usize, usize)> = (0..100 * 50)
                .filter(|i| rgb[i * 3] > 0)
                .map(|i| (i % 100, i / 100))
                .collect();
            assert_eq!(lit.len(), 100, "{anchor:?}");
            assert_eq!(lit.first(), Some(&(x, y)), "{anchor:?}");
            assert_eq!(lit.last(), Some(&(x + 9, y + 9)), "{anchor:?}");
        }
    }

    #[test]
    fn opacity_blends_both_bit_depths() {
        let mark = watermark(mark(10, 10, [255; 4]), (Align::Start, Align::Start), 0.5);
        let mut eight = black(50, 50);
        composite(&mut eight, &mark, OutputColorSpace::SRgb).unwrap();
        assert_eq!(rgb8(&eight)[(5 * 50 + 5) * 3], 128);
        let mut sixteen = FinalImage {
            pixels: FinalPixels::Rgb16(vec![0; 50 * 50 * 3]),
            width: 50,
            height: 50,
        };
        composite(&mut sixteen, &mark, OutputColorSpace::SRgb).unwrap();
        let FinalPixels::Rgb16(rgb) = &sixteen.pixels else {
            panic!("expected rgb16");
        };
        assert_eq!(rgb[(5 * 50 + 5) * 3], 32768);
    }

    #[test]
    fn scaling_does_not_bleed_transparent_colour_into_edges() {
        let rgba: Vec<u8> = (0..64)
            .flat_map(|i| if i % 8 < 4 { [0, 0, 0, 0] } else { [255; 4] })
            .collect();
        let source = Arc::new(WatermarkImage {
            width: 8,
            height: 8,
            rgba,
        });
        let mut image = FinalImage {
            pixels: FinalPixels::Rgb8(vec![128; 50 * 50 * 3]),
            width: 50,
            height: 50,
        };
        let mark = Watermark {
            size: 0.1,
            ..watermark(source, (Align::Start, Align::Start), 1.0)
        };
        composite(&mut image, &mark, OutputColorSpace::SRgb).unwrap();
        assert!(rgb8(&image).iter().all(|&v| v >= 128));
        assert!(rgb8(&image).iter().any(|&v| v > 240));
    }

    #[test]
    fn a_p3_export_converts_the_srgb_mark() {
        let mark = watermark(
            mark(10, 10, [255, 0, 0, 255]),
            (Align::Start, Align::Start),
            1.0,
        );
        let mut image = black(50, 50);
        composite(&mut image, &mark, OutputColorSpace::DisplayP3).unwrap();
        let px = &rgb8(&image)[(5 * 50 + 5) * 3..][..3];
        assert!(px[0] < 250 && px[1] > 40 && px[2] > 20, "{px:?}");
    }
}
