use raw_pipeline::edits::{RetouchMode, RetouchStroke, Vec2f};
use raw_pipeline::{
    cpu, decode, edits::Edits, frame::OutputColorSpace, frame::OutputFormat, frame::RawFrame,
    frame::RenderOptions, gpu,
};
use raw_pipeline_testkit::color::{
    delta_e_2000, linear_rgb_to_lab, luma, mean_and_p95, srgb_to_linear,
};
use raw_pipeline_testkit::fixtures::fixtures;
use raw_pipeline_testkit::gpu::try_renderer;
use raw_pipeline_testkit::lut::{TINT_LUT_ID, tint_luts};
use std::path::PathBuf;

const PSNR_FLOOR_DB: f64 = 34.0;
const SSIM_FLOOR: f64 = 0.998;
const DE2000_MEAN_CEIL: f64 = 2.2;
const DE2000_P95_CEIL: f64 = 3.4;

const VARIANT_FIXTURES: &[&str] = &["Canon_EOS_R6", "Fujifilm_X-T2", "Panasonic_DMC-LX7"];

fn variant_fixtures() -> Vec<PathBuf> {
    fixtures()
        .into_iter()
        .filter(|p| {
            let name = p.file_name().unwrap_or_default().to_string_lossy();
            VARIANT_FIXTURES.iter().any(|n| name.starts_with(n))
        })
        .collect()
}

fn mse(a: &[u8], b: &[u8]) -> f64 {
    let sum: u64 = a
        .iter()
        .zip(b.iter())
        .map(|(&x, &y)| {
            let d = x as i32 - y as i32;
            (d * d) as u64
        })
        .sum();
    sum as f64 / a.len() as f64
}

fn psnr(a: &[u8], b: &[u8]) -> f64 {
    let m = mse(a, b);
    if m <= f64::EPSILON {
        return f64::INFINITY;
    }
    10.0 * (255.0_f64 * 255.0 / m).log10()
}

fn ssim_luma(a: &[u8], b: &[u8]) -> f64 {
    let to_l = |s: &[u8]| -> Vec<f64> { s.chunks_exact(3).map(luma).collect() };
    let la = to_l(a);
    let lb = to_l(b);
    let n = la.len() as f64;
    let mean_a: f64 = la.iter().sum::<f64>() / n;
    let mean_b: f64 = lb.iter().sum::<f64>() / n;
    let mut var_a = 0.0;
    let mut var_b = 0.0;
    let mut cov = 0.0;
    for i in 0..la.len() {
        let da = la[i] - mean_a;
        let db = lb[i] - mean_b;
        var_a += da * da;
        var_b += db * db;
        cov += da * db;
    }
    var_a /= n;
    var_b /= n;
    cov /= n;
    let c1 = (0.01 * 255.0_f64).powi(2);
    let c2 = (0.03 * 255.0_f64).powi(2);
    ((2.0 * mean_a * mean_b + c1) * (2.0 * cov + c2))
        / ((mean_a * mean_a + mean_b * mean_b + c1) * (var_a + var_b + c2))
}

fn srgb_bytes_to_lab(rgb: &[u8]) -> Vec<[f64; 3]> {
    rgb.chunks_exact(3)
        .map(|c| {
            linear_rgb_to_lab([
                srgb_to_linear(c[0]),
                srgb_to_linear(c[1]),
                srgb_to_linear(c[2]),
            ])
        })
        .collect()
}

fn delta_e_stats(a: &[u8], b: &[u8]) -> (f64, f64) {
    let des: Vec<f64> = srgb_bytes_to_lab(a)
        .into_iter()
        .zip(srgb_bytes_to_lab(b))
        .map(|(x, y)| delta_e_2000(x, y))
        .collect();
    mean_and_p95(des)
}

fn render_pair(
    renderer: &gpu::GpuRenderer,
    frame: &RawFrame,
    edits: &Edits,
    opts: &RenderOptions,
) -> Option<(Vec<u8>, Vec<u8>)> {
    let cpu_out = cpu::render(frame, edits, opts).ok()?;
    let gpu_out = renderer.render(frame, edits, opts).ok()?;
    if (cpu_out.width, cpu_out.height) != (gpu_out.width, gpu_out.height) {
        return None;
    }
    Some((cpu_out.bytes, gpu_out.bytes))
}

fn check_parity(label: &str, paths: &[PathBuf], edits: &Edits, opts: &RenderOptions) {
    let Some(renderer) = try_renderer() else {
        return;
    };
    if paths.is_empty() {
        eprintln!("no fixtures; skipping");
        return;
    }
    let mut failed: Vec<String> = Vec::new();
    let mut decoded = 0;
    for p in paths {
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        let bytes = std::fs::read(p).unwrap();
        let Ok(frame) = decode::decode(&bytes) else {
            eprintln!("skip {name}: decode unsupported");
            continue;
        };
        drop(bytes);
        let Some((cpu_rgb, gpu_rgb)) = render_pair(&renderer, &frame, edits, opts) else {
            eprintln!("skip {name}: render failed or dims disagree");
            continue;
        };
        drop(frame);
        decoded += 1;
        let p_db = psnr(&cpu_rgb, &gpu_rgb);
        let s = ssim_luma(&cpu_rgb, &gpu_rgb);
        let (de_mean, de_p95) = delta_e_stats(&cpu_rgb, &gpu_rgb);
        eprintln!(
            "{name} ({label}): PSNR={p_db:.2}dB SSIM={s:.4} ΔE2000 mean={de_mean:.2} p95={de_p95:.2}"
        );
        if p_db < PSNR_FLOOR_DB {
            failed.push(format!("{name}: PSNR {p_db:.2} < {PSNR_FLOOR_DB}"));
        }
        if s < SSIM_FLOOR {
            failed.push(format!("{name}: SSIM {s:.4} < {SSIM_FLOOR}"));
        }
        if de_mean > DE2000_MEAN_CEIL {
            failed.push(format!("{name}: ΔE mean {de_mean:.2} > {DE2000_MEAN_CEIL}"));
        }
        if de_p95 > DE2000_P95_CEIL {
            failed.push(format!("{name}: ΔE p95 {de_p95:.2} > {DE2000_P95_CEIL}"));
        }
    }
    if decoded == 0 {
        eprintln!("no fixtures decoded; skipping");
        return;
    }
    if !failed.is_empty() {
        panic!("{label} parity below floor: {}", failed.join("; "));
    }
}

#[test]
fn gpu_vs_cpu_parity_per_fixture() {
    let opts = RenderOptions {
        max_edge: 512,
        output: OutputFormat::Rgb8,
        ..Default::default()
    };
    check_parity("base", &fixtures(), &Edits::default(), &opts);
}

#[test]
fn gpu_vs_cpu_parity_display_p3() {
    let opts = RenderOptions {
        max_edge: 512,
        output: OutputFormat::Rgb8,
        output_color_space: OutputColorSpace::DisplayP3,
        ..Default::default()
    };
    check_parity("p3", &variant_fixtures(), &Edits::default(), &opts);
}

#[test]
fn gpu_vs_cpu_parity_flat_profile() {
    let opts = RenderOptions {
        max_edge: 512,
        output: OutputFormat::Rgb8,
        ..Default::default()
    };
    let mut edits = Edits::default();
    edits.color.dcp.mode = raw_pipeline::edits::DcpMode::Flat;
    check_parity("flat", &variant_fixtures(), &edits, &opts);
}

#[test]
fn gpu_vs_cpu_parity_with_lut() {
    let opts = RenderOptions {
        max_edge: 512,
        output: OutputFormat::Rgb8,
        luts: tint_luts(),
        ..Default::default()
    };
    let mut edits = Edits::default();
    edits.color.lut_3d.lut_id = Some(TINT_LUT_ID.to_string());
    edits.color.lut_3d.amount = 100.0;
    check_parity("lut", &variant_fixtures(), &edits, &opts);
}

#[test]
fn gpu_vs_cpu_parity_with_noise_reduction() {
    use raw_pipeline::edits::DetailEdits;

    let opts = RenderOptions {
        max_edge: 512,
        output: OutputFormat::Rgb8,
        ..Default::default()
    };
    let edits = Edits {
        detail: DetailEdits {
            luma_nr_amount: 55.0,
            luma_nr_detail: 40.0,
            color_nr_amount: 60.0,
            color_nr_smoothness: 60.0,
            ..Default::default()
        },
        ..Default::default()
    };
    check_parity("nr", &variant_fixtures(), &edits, &opts);
}

fn make_huesat_profile() -> raw_pipeline::DcpProfile {
    use raw_pipeline::dcp::{DcpProfile, HsvEncoding, HueSatMap};
    let hue = 6u32;
    let sat = 4u32;
    let val = 1u32;
    let data = vec![[12.0, 1.15, 1.0]; (hue * sat * val) as usize];
    let fm = [
        [0.9642 * 0.5, 0.9642 * 0.3, 0.9642 * 0.2],
        [0.4, 0.35, 0.25],
        [0.8249 * 0.1, 0.8249 * 0.3, 0.8249 * 0.6],
    ];
    DcpProfile {
        name: None,
        copyright: None,
        unique_camera_model: None,
        calibration_illuminant1: 21,
        calibration_illuminant2: None,
        color_matrix1: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        color_matrix2: None,
        forward_matrix1: Some(fm),
        forward_matrix2: None,
        huesatmap1: Some(std::sync::Arc::new(HueSatMap {
            hue_div: hue,
            sat_div: sat,
            val_div: val,
            encoding: HsvEncoding::Linear,
            data,
        })),
        huesatmap2: None,
        look_table: None,
        tone_curve: None,
        baseline_exposure_offset: 0.0,
        default_black_render: 0,
        embed_policy: 0,
    }
}

#[test]
fn gpu_vs_cpu_parity_with_dcp_huesat() {
    run_dcp_parity(make_huesat_profile(), |_| {});
}

fn tone_profile() -> raw_pipeline::DcpProfile {
    let mut profile = make_huesat_profile();
    profile.tone_curve = Some(std::sync::Arc::new(raw_pipeline::dcp::ToneCurve::new(
        vec![
            [0.0, 0.0],
            [0.25, 0.18],
            [0.5, 0.52],
            [0.75, 0.84],
            [1.0, 1.0],
        ],
    )));
    profile
}

#[test]
fn gpu_vs_cpu_parity_with_dcp_tone() {
    run_dcp_parity(tone_profile(), |_| {});
}

#[test]
fn gpu_vs_cpu_parity_with_dcp_tone_and_curves() {
    use raw_pipeline::edits::{CurvePoint, CurvePoints};
    let pts = |mid: f64| CurvePoints {
        points: vec![
            CurvePoint { x: 0.0, y: 0.0 },
            CurvePoint { x: 0.5, y: mid },
            CurvePoint { x: 1.0, y: 1.0 },
        ],
    };
    run_dcp_parity(tone_profile(), |e| {
        e.basic.curves.composite = pts(0.6);
        e.basic.curves.b = pts(0.42);
        e.basic.curves.luma = pts(0.7);
    });
}

#[test]
fn gpu_vs_cpu_parity_with_dcp_look() {
    use raw_pipeline::dcp::{HsvEncoding, HueSatMap};
    let mut profile = make_huesat_profile();
    let hue = 6u32;
    let sat = 4u32;
    let val = 1u32;
    profile.look_table = Some(std::sync::Arc::new(HueSatMap {
        hue_div: hue,
        sat_div: sat,
        val_div: val,
        encoding: HsvEncoding::Srgb,
        data: vec![[-8.0, 1.1, 0.97]; (hue * sat * val) as usize],
    }));
    run_dcp_parity(profile, |_| {});
}

#[test]
fn gpu_vs_cpu_parity_with_dcp_presence() {
    run_dcp_parity(make_huesat_profile(), |e| {
        e.detail.luma_nr_amount = 55.0;
        e.detail.luma_nr_detail = 40.0;
        e.detail.color_nr_amount = 60.0;
        e.detail.color_nr_smoothness = 60.0;
    });
}

fn run_dcp_parity(profile: raw_pipeline::DcpProfile, tweak: impl FnOnce(&mut Edits)) {
    use raw_pipeline::edits::DcpMode;
    use std::sync::Arc;

    let opts = RenderOptions {
        max_edge: 512,
        output: OutputFormat::Rgb8,
        dcp: Some(Arc::new(profile)),
        ..Default::default()
    };
    let mut edits = Edits::default();
    edits.color.dcp.mode = DcpMode::Profile;
    edits.color.dcp.profile_id = Some("test".to_string());
    tweak(&mut edits);
    check_parity("dcp", &variant_fixtures(), &edits, &opts);
}

fn retouch_edits() -> Edits {
    let stroke = |id: &str, mode: RetouchMode, pts: &[(f32, f32)], src: (f32, f32)| RetouchStroke {
        id: id.to_string(),
        mode,
        points: pts.iter().map(|p| Vec2f { x: p.0, y: p.1 }).collect(),
        radius: 0.12,
        hardness: 0.5,
        opacity: 1.0,
        source: Vec2f { x: src.0, y: src.1 },
        enabled: true,
    };
    Edits {
        retouch: vec![
            stroke("heal", RetouchMode::Heal, &[(0.3, 0.35)], (0.62, 0.55)),
            stroke(
                "clone",
                RetouchMode::Clone,
                &[(0.55, 0.7), (0.68, 0.78)],
                (0.3, 0.4),
            ),
        ],
        ..Default::default()
    }
}

#[test]
fn gpu_vs_cpu_parity_retouch() {
    let opts = RenderOptions {
        max_edge: 512,
        output: OutputFormat::Rgb8,
        ..Default::default()
    };
    check_parity("retouch", &variant_fixtures(), &retouch_edits(), &opts);
}
