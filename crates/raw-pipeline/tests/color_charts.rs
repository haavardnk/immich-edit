use raw_pipeline::{cpu, decode, edits::Edits, frame::RenderOptions};
use raw_pipeline_testkit::color::{delta_e_2000, linear_rgb_to_lab, mean_and_p95, srgb_to_linear};
use raw_pipeline_testkit::fixtures::fixture_path;
use raw_pipeline_testkit::render::decode_jpeg_rgb;

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

#[test]
fn color_chart_delta_e_against_xrite() {
    let mut failures: Vec<String> = Vec::new();
    for chart in CHARTS {
        let path = fixture_path(chart.name);
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
        let (rgb, w, h) = decode_jpeg_rgb(&out.bytes);
        let mut sampled_lin: Vec<[f64; 3]> = Vec::with_capacity(24);
        let mut ref_lin: Vec<[f64; 3]> = Vec::with_capacity(24);
        for row in 0..4 {
            for col in 0..6 {
                let (cx, cy) = patch_center(chart.corners, col, row);
                let sampled = sample_patch(&rgb, w, h, cx, cy);
                let idx = row * 6 + col;
                let r = XRITE_SRGB[idx];
                sampled_lin.push([
                    srgb_to_linear(sampled[0]),
                    srgb_to_linear(sampled[1]),
                    srgb_to_linear(sampled[2]),
                ]);
                ref_lin.push([
                    srgb_to_linear(r[0]),
                    srgb_to_linear(r[1]),
                    srgb_to_linear(r[2]),
                ]);
            }
        }
        let mut num = 0.0f64;
        let mut den = 0.0f64;
        for (s, r) in sampled_lin.iter().zip(&ref_lin) {
            for ch in 0..3 {
                num += s[ch] * r[ch];
                den += s[ch] * s[ch];
            }
        }
        let scale = if den > 0.0 { num / den } else { 1.0 };
        let des: Vec<f64> = sampled_lin
            .iter()
            .zip(&ref_lin)
            .map(|(s, r)| {
                let scaled = [s[0] * scale, s[1] * scale, s[2] * scale];
                delta_e_2000(linear_rgb_to_lab(scaled), linear_rgb_to_lab(*r))
            })
            .collect();
        let (mean, p95) = mean_and_p95(des);
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
