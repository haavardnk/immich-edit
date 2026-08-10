use raw_pipeline::cpu::run_pipeline_ops;
use raw_pipeline::edits::{DetailEdits, Edits};
use raw_pipeline::ops::{LinearImage, OpContext, OpScratch, RenderContext};

const W: usize = 128;
const H: usize = 128;

fn ctx() -> OpContext {
    OpContext {
        render: RenderContext {
            wb_coeffs: [1.0, 1.0, 1.0, 1.0],
            cam_to_srgb: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            is_raw: true,
            capture_sigma: None,
            preview_mode: raw_pipeline::frame::PreviewMode::None,
            dcp: None,
        },
        scratch: OpScratch::default(),
    }
}

fn noisy_image() -> LinearImage {
    let mut rgb = Vec::with_capacity(W * H * 3);
    let mut seed: u32 = 0x1234_5678;
    for y in 0..H {
        for x in 0..W {
            let u = x as f32 / W as f32;
            let v = y as f32 / H as f32;
            let base = [0.25 + 0.4 * u, 0.3 + 0.3 * v, 0.35 + 0.2 * (u * v)];
            for c in base {
                seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                let noise = (seed >> 8) as f32 / 8_388_608.0 - 1.0;
                rgb.push((c + 0.06 * noise).clamp(0.0, 1.0));
            }
        }
    }
    LinearImage::new(rgb, W, H)
}

fn high_freq(img: &LinearImage, plane: impl Fn(&[f32]) -> f32) -> f64 {
    let px = |x: usize, y: usize| -> f32 {
        let i = (y * img.width + x) * 3;
        plane(&img.rgb[i..i + 3])
    };
    let mut sum = 0.0_f64;
    let mut n = 0_u64;
    for y in 1..img.height - 1 {
        for x in 1..img.width - 1 {
            let lap = 4.0 * px(x, y) - px(x - 1, y) - px(x + 1, y) - px(x, y - 1) - px(x, y + 1);
            sum += lap.abs() as f64;
            n += 1;
        }
    }
    sum / n as f64
}

fn luma(p: &[f32]) -> f32 {
    0.299 * p[0] + 0.587 * p[1] + 0.114 * p[2]
}

fn chroma(p: &[f32]) -> f32 {
    p[2] - luma(p)
}

fn render(base: &LinearImage, edits: &Edits) -> LinearImage {
    let mut img = LinearImage::new(base.rgb.clone(), base.width, base.height);
    let rasters = raw_pipeline::empty_rasters();
    run_pipeline_ops(&mut img, &ctx(), edits, &rasters, None).unwrap();
    img
}

type Plane = fn(&[f32]) -> f32;

#[test]
fn cpu_pipeline_runs_noise_reduction() {
    let base = noisy_image();
    let plain = render(&base, &Edits::default());
    let cases: [(&str, DetailEdits, Plane); 2] = [
        (
            "luma_nr",
            DetailEdits {
                luma_nr_amount: 80.0,
                luma_nr_detail: 20.0,
                ..Default::default()
            },
            luma,
        ),
        (
            "color_nr",
            DetailEdits {
                color_nr_amount: 80.0,
                color_nr_detail: 20.0,
                color_nr_smoothness: 60.0,
                ..Default::default()
            },
            chroma,
        ),
    ];
    for (label, detail, plane) in cases {
        let denoised = render(
            &base,
            &Edits {
                detail,
                ..Default::default()
            },
        );
        if denoised.rgb == plain.rgb {
            panic!("{label}: CPU pipeline left the image untouched");
        }
        let before = high_freq(&plain, plane);
        let after = high_freq(&denoised, plane);
        if after >= before * 0.9 {
            panic!("{label}: high-frequency energy {after} not reduced from {before}");
        }
    }
}
