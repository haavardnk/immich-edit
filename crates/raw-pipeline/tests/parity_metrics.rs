use raw_pipeline::{cpu, decode, edits::Edits, frame::RawFrame, frame::RenderOptions, gpu};
use std::path::{Path, PathBuf};

const RAW_EXTS: &[&str] = &[
    "arw", "cr2", "cr3", "crw", "dng", "erf", "gpr", "iiq", "mrw", "nef", "nrw", "orf", "pef",
    "raf", "raw", "rw2", "rwl", "sr2", "srw", "x3f",
];

const PSNR_FLOOR_DB: f64 = 26.0;
const SSIM_FLOOR: f64 = 0.97;
const DE2000_MEAN_CEIL: f64 = 5.0;
const DE2000_P95_CEIL: f64 = 12.0;

fn fixtures() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .map(|e| RAW_EXTS.contains(&e.to_ascii_lowercase().as_str()))
                .unwrap_or(false)
        })
        .collect();
    paths.sort();
    paths
}

fn try_renderer() -> Option<gpu::GpuRenderer> {
    match gpu::GpuRenderer::new() {
        Ok(r) => Some(r),
        Err(e) => {
            eprintln!("skip: gpu init failed ({e})");
            None
        }
    }
}

fn decode_jpeg_rgb(jpeg: &[u8]) -> (Vec<u8>, usize, usize) {
    let img: turbojpeg::Image<Vec<u8>> =
        turbojpeg::decompress(jpeg, turbojpeg::PixelFormat::RGB).unwrap();
    (img.pixels, img.width, img.height)
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
    let to_l = |s: &[u8]| -> Vec<f64> {
        s.chunks_exact(3)
            .map(|c| 0.2126 * c[0] as f64 + 0.7152 * c[1] as f64 + 0.0722 * c[2] as f64)
            .collect()
    };
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

fn srgb_to_linear(c: u8) -> f64 {
    let x = c as f64 / 255.0;
    if x <= 0.04045 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}

fn rgb_to_lab(rgb: &[u8]) -> Vec<[f64; 3]> {
    let xn = 0.95047;
    let yn = 1.0;
    let zn = 1.08883;
    let f = |t: f64| -> f64 {
        let d = 6.0 / 29.0;
        if t > d * d * d {
            t.cbrt()
        } else {
            t / (3.0 * d * d) + 4.0 / 29.0
        }
    };
    rgb.chunks_exact(3)
        .map(|c| {
            let r = srgb_to_linear(c[0]);
            let g = srgb_to_linear(c[1]);
            let b = srgb_to_linear(c[2]);
            let x = 0.4124564 * r + 0.3575761 * g + 0.1804375 * b;
            let y = 0.2126729 * r + 0.7151522 * g + 0.0721750 * b;
            let z = 0.0193339 * r + 0.1191920 * g + 0.9503041 * b;
            let fx = f(x / xn);
            let fy = f(y / yn);
            let fz = f(z / zn);
            [116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)]
        })
        .collect()
}

fn delta_e_2000(lab1: [f64; 3], lab2: [f64; 3]) -> f64 {
    let [l1, a1, b1] = lab1;
    let [l2, a2, b2] = lab2;
    let c1 = (a1 * a1 + b1 * b1).sqrt();
    let c2 = (a2 * a2 + b2 * b2).sqrt();
    let c_bar = 0.5 * (c1 + c2);
    let c7 = c_bar.powi(7);
    let g = 0.5 * (1.0 - (c7 / (c7 + 25f64.powi(7))).sqrt());
    let a1p = (1.0 + g) * a1;
    let a2p = (1.0 + g) * a2;
    let c1p = (a1p * a1p + b1 * b1).sqrt();
    let c2p = (a2p * a2p + b2 * b2).sqrt();
    let h1p = if b1 == 0.0 && a1p == 0.0 {
        0.0
    } else {
        b1.atan2(a1p).to_degrees().rem_euclid(360.0)
    };
    let h2p = if b2 == 0.0 && a2p == 0.0 {
        0.0
    } else {
        b2.atan2(a2p).to_degrees().rem_euclid(360.0)
    };
    let dlp = l2 - l1;
    let dcp = c2p - c1p;
    let dhp = if c1p * c2p == 0.0 {
        0.0
    } else {
        let diff = h2p - h1p;
        if diff > 180.0 {
            diff - 360.0
        } else if diff < -180.0 {
            diff + 360.0
        } else {
            diff
        }
    };
    let dhp_rad = (dhp / 2.0).to_radians();
    let dhp_term = 2.0 * (c1p * c2p).sqrt() * dhp_rad.sin();
    let l_bar = 0.5 * (l1 + l2);
    let c_bar_p = 0.5 * (c1p + c2p);
    let h_bar_p = if c1p * c2p == 0.0 {
        h1p + h2p
    } else if (h1p - h2p).abs() <= 180.0 {
        0.5 * (h1p + h2p)
    } else if h1p + h2p < 360.0 {
        0.5 * (h1p + h2p + 360.0)
    } else {
        0.5 * (h1p + h2p - 360.0)
    };
    let t = 1.0 - 0.17 * (h_bar_p - 30.0).to_radians().cos()
        + 0.24 * (2.0 * h_bar_p).to_radians().cos()
        + 0.32 * (3.0 * h_bar_p + 6.0).to_radians().cos()
        - 0.20 * (4.0 * h_bar_p - 63.0).to_radians().cos();
    let dtheta = 30.0 * (-((h_bar_p - 275.0) / 25.0).powi(2)).exp();
    let rc = 2.0 * (c_bar_p.powi(7) / (c_bar_p.powi(7) + 25f64.powi(7))).sqrt();
    let sl = 1.0 + (0.015 * (l_bar - 50.0).powi(2)) / (20.0 + (l_bar - 50.0).powi(2)).sqrt();
    let sc = 1.0 + 0.045 * c_bar_p;
    let sh = 1.0 + 0.015 * c_bar_p * t;
    let rt = -(2.0 * dtheta).to_radians().sin() * rc;
    ((dlp / sl).powi(2)
        + (dcp / sc).powi(2)
        + (dhp_term / sh).powi(2)
        + rt * (dcp / sc) * (dhp_term / sh))
        .sqrt()
}

fn delta_e_stats(a: &[u8], b: &[u8]) -> (f64, f64) {
    let lab_a = rgb_to_lab(a);
    let lab_b = rgb_to_lab(b);
    let mut des: Vec<f64> = lab_a
        .iter()
        .zip(lab_b.iter())
        .map(|(&x, &y)| delta_e_2000(x, y))
        .collect();
    let mean = des.iter().sum::<f64>() / des.len() as f64;
    des.sort_by(|x, y| x.partial_cmp(y).unwrap());
    let p95 = des[((des.len() as f64) * 0.95) as usize];
    (mean, p95)
}

fn render_pair(
    renderer: &gpu::GpuRenderer,
    frame: &RawFrame,
    edits: &Edits,
    opts: &RenderOptions,
) -> Option<(Vec<u8>, Vec<u8>)> {
    let cpu_out = cpu::render(frame, edits, opts).ok()?;
    let gpu_out = renderer.render(frame, edits, opts).ok()?;
    let (cpu_rgb, cw, ch) = decode_jpeg_rgb(&cpu_out.bytes);
    let (gpu_rgb, gw, gh) = decode_jpeg_rgb(&gpu_out.bytes);
    if (cw, ch) != (gw, gh) {
        return None;
    }
    Some((cpu_rgb, gpu_rgb))
}

#[test]
fn gpu_vs_cpu_parity_per_fixture() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let paths = fixtures();
    if paths.is_empty() {
        eprintln!("no fixtures; skipping");
        return;
    }
    let opts = RenderOptions {
        max_edge: 512,
        ..Default::default()
    };
    let mut failed: Vec<String> = Vec::new();
    let mut decoded = 0;
    for p in &paths {
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        let bytes = std::fs::read(p).unwrap();
        let frame = match decode::decode(&bytes) {
            Ok(f) => f,
            Err(_) => {
                eprintln!("skip {name}: decode unsupported");
                continue;
            }
        };
        let Some((cpu_rgb, gpu_rgb)) = render_pair(&renderer, &frame, &Edits::default(), &opts)
        else {
            eprintln!("skip {name}: render or jpeg decode failed");
            continue;
        };
        decoded += 1;
        let p_db = psnr(&cpu_rgb, &gpu_rgb);
        let s = ssim_luma(&cpu_rgb, &gpu_rgb);
        let (de_mean, de_p95) = delta_e_stats(&cpu_rgb, &gpu_rgb);
        eprintln!("{name}: PSNR={p_db:.2}dB SSIM={s:.4} ΔE2000 mean={de_mean:.2} p95={de_p95:.2}");
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
        panic!("parity below floor: {}", failed.join("; "));
    }
}
