pub fn luma(rgb: &[u8]) -> f64 {
    0.2126 * rgb[0] as f64 + 0.7152 * rgb[1] as f64 + 0.0722 * rgb[2] as f64
}

pub fn srgb_to_linear(c: u8) -> f64 {
    let x = c as f64 / 255.0;
    if x <= 0.04045 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}

pub fn linear_rgb_to_lab(lin: [f64; 3]) -> [f64; 3] {
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
    let [r, g, b] = lin;
    let x = 0.4124564 * r + 0.3575761 * g + 0.1804375 * b;
    let y = 0.2126729 * r + 0.7151522 * g + 0.0721750 * b;
    let z = 0.0193339 * r + 0.1191920 * g + 0.9503041 * b;
    let fx = f(x / xn);
    let fy = f(y / yn);
    let fz = f(z / zn);
    [116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)]
}

pub fn delta_e_2000(lab1: [f64; 3], lab2: [f64; 3]) -> f64 {
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

pub fn mean_and_p95(mut values: Vec<f64>) -> (f64, f64) {
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    values.sort_by(f64::total_cmp);
    let p95 = values[(values.len() as f64 * 0.95) as usize];
    (mean, p95)
}
