#![cfg(target_vendor = "apple")]

use std::sync::Arc;

use raw_pipeline::CpuRenderer;
use raw_pipeline::cpu::{estimate_render, estimate_source};
use raw_pipeline::edits::{
    CropRect, Edits, MaskComponentKind, MaskLayer, MaskedEdits, RetouchMode, Vec2f,
};
use raw_pipeline::encode::{encode_from_rgb8, encode_from_rgb16, working_bytes};
use raw_pipeline::frame::OutputColorSpace::SRgb;
use raw_pipeline::frame::{
    Align, BitDepth, JpegSubsampling, OutputFormat, OutputSharpen, PngCompression, PreviewMode,
    RawFrame, RenderOptions, SharpenLevel, SharpenMedia, TiffCompression, Watermark,
    WatermarkAnchor, WatermarkImage,
};
use raw_pipeline_testkit::depth::{depth_rasters, lens_blur_edits};
use raw_pipeline_testkit::frames::synthetic_bayer_frame;
use raw_pipeline_testkit::masks::layer;
use raw_pipeline_testkit::memory::{Counting, check, finish, heap_peak, measuring};
use raw_pipeline_testkit::retouch::stroke;

const XTRANS: &str = "GGRGGBGGBGGRBRGRBGGGBGGRGGRGGBRBGBRG";

#[global_allocator]
static COUNTING: Counting = Counting;

fn sharp_mosaic(width: usize, height: usize, cfa: &str) -> RawFrame {
    let mut frame = synthetic_bayer_frame(width, height, cfa);
    frame.data = (0..width * height)
        .map(|i| {
            let (x, y) = (i % width, i / width);
            let hash = (x.wrapping_mul(73_856_093) ^ y.wrapping_mul(19_349_663)) % 1000;
            0.2 + 0.5 * (x as f32 / width as f32) + hash as f32 / 2_000.0
        })
        .collect();
    frame.meta.is_raw = true;
    frame.meta.capture_sigma = Some(0.8);
    frame
}

fn masks() -> Vec<MaskLayer> {
    let range = MaskComponentKind::LumaRange {
        min: 0.5,
        max: 1.0,
        softness: 0.2,
    };
    let gradient = MaskComponentKind::Linear {
        p0: Vec2f { x: 0.0, y: 0.0 },
        p1: Vec2f { x: 1.0, y: 1.0 },
        feather: 0.2,
    };
    let darker = MaskedEdits {
        exposure_ev: Some(-0.5),
        ..Default::default()
    };
    let sharpened = MaskedEdits {
        exposure_ev: Some(0.3),
        sharpen: Some(40.0),
        ..Default::default()
    };
    vec![
        layer("highlights", range, darker),
        layer("gradient", gradient, sharpened),
    ]
}

fn heavy() -> Edits {
    let mut edits = Edits::default();
    edits.detail.luma_nr_amount = 40.0;
    edits.detail.color_nr_amount = 30.0;
    edits.basic.dehaze = 30.0;
    edits.basic.clarity = 25.0;
    edits.basic.texture = 20.0;
    edits.tone.shadows = 30.0;
    edits.geometry.rotate_angle = 3.0;
    edits
}

fn masked() -> Edits {
    let mut edits = lens_blur_edits(40.0, Default::default(), 0.0);
    edits.basic.clarity = 25.0;
    edits.tone.shadows = 30.0;
    edits.masks = masks();
    edits
}

fn retouched() -> Edits {
    let heal = stroke(
        RetouchMode::Heal,
        &[(0.3, 0.4), (0.35, 0.45)],
        0.05,
        (0.6, 0.4),
    );
    let mut edits = Edits {
        retouch: vec![heal],
        ..Edits::default()
    };
    edits.basic.texture = 20.0;
    edits.detail.luma_nr_amount = 40.0;
    edits
}

fn portrait() -> Edits {
    let mut edits = Edits::default();
    edits.geometry.rotate = 90;
    edits.geometry.crop = Some(CropRect {
        x: 0.2,
        y: 0.2,
        w: 0.6,
        h: 0.6,
    });
    edits.detail.sharpen_masking = 30.0;
    edits
}

fn unsharpened() -> Edits {
    let mut edits = Edits::default();
    edits.detail.sharpen_amount = Some(0.0);
    edits.detail.capture_sharpen = false;
    edits
}

fn export(output: OutputFormat) -> RenderOptions {
    RenderOptions {
        max_edge: u32::MAX,
        quality: true,
        output,
        rasters: depth_rasters(),
        ..RenderOptions::default()
    }
}

fn jpeg() -> OutputFormat {
    OutputFormat::Jpeg {
        quality: 95,
        subsampling: JpegSubsampling::Chroma444,
    }
}

fn finished_jpeg() -> RenderOptions {
    let (width, height) = (3000, 2000);
    RenderOptions {
        max_edge: 8000,
        enlarge: true,
        output_sharpen: Some(OutputSharpen {
            media: SharpenMedia::Glossy { ppi: 300 },
            level: SharpenLevel::High,
        }),
        watermark: Some(Watermark {
            image: Arc::new(WatermarkImage {
                width,
                height,
                rgba: vec![200; (width * height * 4) as usize],
            }),
            size: 1.0,
            opacity: 0.5,
            anchor: WatermarkAnchor {
                x: Align::Center,
                y: Align::Center,
            },
            inset: 0.0,
        }),
        ..export(jpeg())
    }
}

fn preview() -> RenderOptions {
    RenderOptions {
        max_edge: 2048,
        output: jpeg(),
        rasters: depth_rasters(),
        ..RenderOptions::default()
    }
}

#[derive(Clone, Copy)]
enum Output {
    Image,
    Source,
}

struct Case {
    name: &'static str,
    edits: Edits,
    options: RenderOptions,
    output: Output,
    xtrans_portrait: bool,
}

impl Case {
    fn frame(&self, (width, height): (usize, usize)) -> RawFrame {
        if !self.xtrans_portrait {
            return sharp_mosaic(width, height, "RGGB");
        }
        let mut frame = sharp_mosaic(width, height + 2, XTRANS);
        frame.meta.orientation = (true, false, true);
        frame
    }
}

fn case(name: &'static str, edits: Edits, options: RenderOptions) -> Case {
    Case {
        name,
        edits,
        options,
        output: Output::Image,
        xtrans_portrait: false,
    }
}

fn source(name: &'static str, options: RenderOptions) -> Case {
    Case {
        output: Output::Source,
        ..case(name, heavy(), options)
    }
}

fn cases() -> [Case; 9] {
    let png16 = OutputFormat::Png {
        bit_depth: BitDepth::Sixteen,
        compression: PngCompression::Fast,
    };
    let tiff = OutputFormat::Tiff {
        bit_depth: BitDepth::Eight,
        compression: TiffCompression::None,
    };
    let mask_weight = RenderOptions {
        preview_mode: PreviewMode::MaskWeight {
            layer_id: "highlights".into(),
        },
        ..preview()
    };
    let tile = RenderOptions {
        roi: Some(CropRect {
            x: 0.55,
            y: 0.4,
            w: 0.3,
            h: 0.3,
        }),
        ..preview()
    };
    [
        case("heavy 16-bit png export", heavy(), export(png16)),
        case("masked tiff export", masked(), export(tiff)),
        case("retouched jpeg export", retouched(), export(jpeg())),
        case("enlarged watermarked jpeg", unsharpened(), finished_jpeg()),
        case("cacheable masked preview", masked(), preview()),
        case("mask weight preview", masked(), mask_weight),
        Case {
            xtrans_portrait: true,
            ..case("x-trans portrait tiff export", portrait(), export(tiff))
        },
        source("source", preview()),
        source("tile source", tile),
    ]
}

fn run(case: &Case, frame: &RawFrame) -> u64 {
    let renderer = CpuRenderer::new();
    let (edits, options) = (&case.edits, &case.options);
    heap_peak(|| {
        let rendered = match case.output {
            Output::Image => renderer.render(frame, edits, options).map(drop),
            Output::Source => renderer
                .render_source(frame, edits, options, None)
                .map(drop),
        };
        if let Err(e) = rendered {
            panic!("{}: the measured render failed: {e}", case.name);
        }
    })
}

fn estimate(case: &Case, frame: &RawFrame) -> u64 {
    let estimated = match case.output {
        Output::Image => estimate_render(frame, &case.edits, &case.options),
        Output::Source => estimate_source(frame, &case.edits, &case.options),
    };
    estimated.unwrap_or_else(|e| panic!("{}: estimate failed: {e}", case.name))
}

#[test]
fn the_estimate_covers_the_measured_peak_of_cpu_renders() {
    let _measuring = measuring();
    let mut failures = Vec::new();
    for case in cases() {
        run(&case, &case.frame((512, 342)));
        let frame = case.frame((4240, 2832));
        let measured = run(&case, &frame);
        check(case.name, measured, estimate(&case, &frame), &mut failures);
    }
    finish(failures);
}

fn noise<T>(len: usize, sample: impl Fn(u32) -> T) -> Vec<T> {
    let mut state = 0x9e37_79b9u32;
    (0..len)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            sample(state)
        })
        .collect()
}

fn rust_encoded_formats() -> Vec<OutputFormat> {
    let mut formats = vec![OutputFormat::Rgb8];
    for bit_depth in [BitDepth::Eight, BitDepth::Sixteen] {
        for compression in [PngCompression::Fast, PngCompression::Default] {
            formats.push(OutputFormat::Png {
                bit_depth,
                compression,
            });
        }
        for compression in [
            TiffCompression::None,
            TiffCompression::Lzw,
            TiffCompression::Deflate,
        ] {
            formats.push(OutputFormat::Tiff {
                bit_depth,
                compression,
            });
        }
    }
    formats
}

#[test]
fn the_encoder_estimate_covers_the_measured_peak_of_each_rust_encoder() {
    let _measuring = measuring();
    let (w, h) = (4240u32, 2832u32);
    let len = (w * h * 3) as usize;
    let rgb8 = noise(len, |v| (v >> 24) as u8);
    let rgb16 = noise(len, |v| (v >> 16) as u16);
    let mut failures = Vec::new();
    for format in rust_encoded_formats() {
        let measured = heap_peak(|| {
            let encoded = match format.bit_depth() {
                BitDepth::Sixteen => encode_from_rgb16(&rgb16, w, h, &format, SRgb, None),
                BitDepth::Eight => encode_from_rgb8(&rgb8, w, h, &format, SRgb, None),
            };
            if let Err(e) = encoded {
                panic!("{format:?}: encode failed: {e}");
            }
        });
        let estimate = working_bytes(&format, (w, h));
        check(&format!("{format:?}"), measured, estimate, &mut failures);
    }
    finish(failures);
}
