use raw_pipeline::{cpu, decode, edits::Edits, frame::RenderOptions};

const XRITE_SRGB: [[u8; 3]; 24] = [
    [115, 82, 68],
    [194, 150, 130],
    [98, 122, 157],
    [87, 108, 67],
    [133, 128, 177],
    [103, 189, 170],
    [214, 126, 44],
    [80, 91, 166],
    [193, 90, 99],
    [94, 60, 108],
    [157, 188, 64],
    [224, 163, 46],
    [56, 61, 150],
    [70, 148, 73],
    [175, 54, 60],
    [231, 199, 31],
    [187, 86, 149],
    [8, 133, 161],
    [243, 243, 242],
    [200, 200, 200],
    [160, 160, 160],
    [122, 122, 121],
    [85, 85, 85],
    [52, 52, 52],
];

const DE2000_MEAN_CEIL: f64 = 10.0;
const DE2000_P95_CEIL: f64 = 18.0;

struct Chart {
    name: &'static str,
    corners: [(f32, f32); 4],
}

const CHARTS: &[Chart] = &[
    Chart {
        name: "Pentax_K10D_12bit_12bit_compressed_3-2.pef",
        corners: [
            (175.0, 200.0),
            (340.0, 203.0),
            (340.0, 302.0),
            (175.0, 300.0),
        ],
    },
    Chart {
        name: "Nikon_D2H_12bit_12bit_compressed_Lossy_type_1_3-2.nef",
        corners: [
            (140.0, 115.0),
            (380.0, 100.0),
            (380.0, 230.0),
            (140.0, 245.0),
        ],
    },
];

fn bilerp(c: [(f32, f32); 4], u: f32, v: f32) -> (f32, f32) {
    let tl = c[0];
    let tr = c[1];
    let br = c[2];
    let bl = c[3];
    let top_x = tl.0 + (tr.0 - tl.0) * u;
    let top_y = tl.1 + (tr.1 - tl.1) * u;
    let bot_x = bl.0 + (br.0 - bl.0) * u;
    let bot_y = bl.1 + (br.1 - bl.1) * u;
    (top_x + (bot_x - top_x) * v, top_y + (bot_y - top_y) * v)
}

fn patch_center(c: [(f32, f32); 4], col: usize, row: usize) -> (f32, f32) {
    let u = (col as f32 + 0.5) / 6.0;
    let v = (row as f32 + 0.5) / 4.0;
    bilerp(c, u, v)
}

fn sample_patch(rgb: &[u8], w: usize, h: usize, cx: f32, cy: f32) -> [u8; 3] {
    let (px, py) = (cx as i32, cy as i32);
    let mut sum: [u32; 3] = [0, 0, 0];
    let mut n: u32 = 0;
    for dy in -2i32..=2 {
        for dx in -2i32..=2 {
            let x = px + dx;
            let y = py + dy;
            if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
                continue;
            }
            let i = (y as usize * w + x as usize) * 3;
            sum[0] += rgb[i] as u32;
            sum[1] += rgb[i + 1] as u32;
            sum[2] += rgb[i + 2] as u32;
            n += 1;
        }
    }
    [(sum[0] / n) as u8, (sum[1] / n) as u8, (sum[2] / n) as u8]
}

fn srgb_to_linear(c: u8) -> f64 {
    let x = c as f64 / 255.0;
    if x <= 0.04045 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}

fn rgb_to_lab(c: [u8; 3]) -> [f64; 3] {
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

#[test]
fn color_chart_delta_e_against_xrite() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut failures: Vec<String> = Vec::new();
    for chart in CHARTS {
        let path = dir.join(chart.name);
        if !path.exists() {
            eprintln!("skip: {} (fixture missing)", chart.name);
            continue;
        }
        let bytes = std::fs::read(&path).unwrap();
        let frame = decode::decode(&bytes).unwrap();
        let opts = RenderOptions {
            max_edge: 512,
            ..Default::default()
        };
        let out = cpu::render(&frame, &Edits::default(), &opts).unwrap();
        let img: turbojpeg::Image<Vec<u8>> =
            turbojpeg::decompress(&out.bytes, turbojpeg::PixelFormat::RGB).unwrap();
        let rgb = img.pixels;
        let (w, h) = (img.width, img.height);
        let mut des: Vec<f64> = Vec::with_capacity(24);
        for row in 0..4 {
            for col in 0..6 {
                let (cx, cy) = patch_center(chart.corners, col, row);
                let sampled = sample_patch(&rgb, w, h, cx, cy);
                let idx = row * 6 + col;
                let lab_s = rgb_to_lab(sampled);
                let lab_r = rgb_to_lab(XRITE_SRGB[idx]);
                des.push(delta_e_2000(lab_s, lab_r));
            }
        }
        let mean = des.iter().sum::<f64>() / des.len() as f64;
        let mut sorted = des.clone();
        sorted.sort_by(|x, y| x.partial_cmp(y).unwrap());
        let p95 = sorted[(sorted.len() as f64 * 0.95) as usize];
        eprintln!("{}: mean ΔE2000={:.2} p95={:.2}", chart.name, mean, p95);
        if mean > DE2000_MEAN_CEIL {
            failures.push(format!(
                "{}: mean ΔE2000 {:.2} > ceiling {:.2}",
                chart.name, mean, DE2000_MEAN_CEIL
            ));
        }
        if p95 > DE2000_P95_CEIL {
            failures.push(format!(
                "{}: p95 ΔE2000 {:.2} > ceiling {:.2}",
                chart.name, p95, DE2000_P95_CEIL
            ));
        }
    }
    if !failures.is_empty() {
        panic!("color chart regressions:\n  {}", failures.join("\n  "));
    }
}
