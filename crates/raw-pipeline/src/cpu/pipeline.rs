use crate::cpu::transform;
use crate::edits::Edits;
use crate::encode::encode_jpeg;
use crate::frame::{RawFrame, RenderOptions, RenderedImage};
use crate::histogram::Histogram;
use crate::ops::LinearImage;
use crate::ops::{OpContext, default_registry};

pub fn render(
    frame: &RawFrame,
    edits: &Edits,
    options: &RenderOptions,
) -> crate::PipelineResult<RenderedImage> {
    let edits = edits.clamped();

    let (rgb, w, h) =
        transform::apply_orientation(frame.data.clone(), frame.width, frame.height, frame.orientation);

    let mut image = LinearImage::new(rgb, w, h);
    let ctx = OpContext {
        wb_coeffs: frame.wb_coeffs,
    };

    let registry = default_registry();
    for op in registry.active(&edits) {
        op.apply_cpu(&mut image, &ctx, &edits)?;
    }

    let (rgb, w, h) = transform::resize(&image.rgb, image.width, image.height, options.max_edge);

    let histogram = Histogram::from_rgb(&rgb, w, h);

    let mut srgb = rgb;
    linear_to_srgb(&mut srgb);

    let rgb_u8: Vec<u8> = srgb
        .iter()
        .map(|&v| (v.clamp(0.0, 1.0) * 255.0) as u8)
        .collect();
    let jpeg = encode_jpeg(&rgb_u8, w as u32, h as u32, 85)?;

    Ok(RenderedImage {
        jpeg,
        histogram,
        width: w as u32,
        height: h as u32,
        renderer: "cpu".into(),
    })
}

fn linear_to_srgb(rgb: &mut [f32]) {
    for v in rgb.iter_mut() {
        *v = if *v <= 0.0031308 {
            *v * 12.92
        } else {
            1.055 * v.powf(1.0 / 2.4) - 0.055
        };
    }
}
