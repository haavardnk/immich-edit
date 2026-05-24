use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use raw_pipeline::cpu::run_pipeline_ops;
use raw_pipeline::edits::{BasicEdits, CurvePoint, CurvePoints, Edits, ToneEdits};
use raw_pipeline::ops::{LinearImage, OpContext};

fn make_image(w: usize, h: usize) -> LinearImage {
    let mut rgb = Vec::with_capacity(w * h * 3);
    for y in 0..h {
        for x in 0..w {
            let u = x as f32 / w as f32;
            let v = y as f32 / h as f32;
            rgb.push(0.2 + 0.6 * u);
            rgb.push(0.15 + 0.7 * v);
            rgb.push(0.25 + 0.5 * (u * v));
        }
    }
    LinearImage::new(rgb, w, h)
}

fn ctx() -> OpContext {
    OpContext {
        wb_coeffs: [2.1, 1.0, 1.45, 1.0],
        cam_to_srgb: [
            [1.85, -0.65, -0.20],
            [-0.20, 1.45, -0.25],
            [0.02, -0.45, 1.43],
        ],
        is_raw: true,
        preview_mode: raw_pipeline::frame::PreviewMode::None,
    }
}

fn edits_identity() -> Edits {
    Edits::default()
}

fn edits_typical() -> Edits {
    Edits {
        basic: BasicEdits {
            exposure_ev: 0.4,
            contrast: 25.0,
            saturation: 10.0,
            curves: CurvePoints {
                points: vec![
                    CurvePoint { x: 0.0, y: 0.05 },
                    CurvePoint { x: 0.5, y: 0.55 },
                    CurvePoint { x: 1.0, y: 0.98 },
                ],
            },
            ..Default::default()
        },
        ..Default::default()
    }
}

fn edits_full() -> Edits {
    Edits {
        basic: BasicEdits {
            exposure_ev: 0.6,
            contrast: 30.0,
            saturation: 15.0,
            vibrance: 25.0,
            wb_temp: 12.0,
            wb_tint: -5.0,
            texture: 20.0,
            clarity: 15.0,
            dehaze: 10.0,
            curves: CurvePoints {
                points: vec![
                    CurvePoint { x: 0.0, y: 0.02 },
                    CurvePoint { x: 0.3, y: 0.28 },
                    CurvePoint { x: 0.7, y: 0.75 },
                    CurvePoint { x: 1.0, y: 0.96 },
                ],
            },
        },
        tone: ToneEdits {
            highlights: -30.0,
            shadows: 25.0,
            blacks: 10.0,
            whites: -15.0,
        },
        ..Default::default()
    }
}

fn bench_ops(c: &mut Criterion) {
    const W: usize = 1024;
    const H: usize = 1024;
    let base = make_image(W, H);
    let ctx = ctx();
    let mut group = c.benchmark_group("cpu_ops");
    group.throughput(Throughput::Elements((W * H) as u64));
    for (label, edits) in [
        ("identity", edits_identity()),
        ("typical", edits_typical()),
        ("full", edits_full()),
    ] {
        group.bench_with_input(BenchmarkId::from_parameter(label), &edits, |b, edits| {
            b.iter_batched(
                || LinearImage::new(base.rgb.clone(), base.width, base.height),
                |mut img| run_pipeline_ops(&mut img, &ctx, edits, None).unwrap(),
                criterion::BatchSize::LargeInput,
            );
        });
    }
    group.finish();
}

criterion_group!(benches, bench_ops);
criterion_main!(benches);
