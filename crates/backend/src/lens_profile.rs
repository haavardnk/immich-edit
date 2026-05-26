use crate::immich::dto::ExifInfo;
use lensfun::{
    CalibDistortion, CalibTca, CalibVignetting, Database, DistortionModel, Lens, TcaModel,
    VignettingModel,
};
use std::sync::OnceLock;

static DB: OnceLock<Option<Database>> = OnceLock::new();

fn db() -> Option<&'static Database> {
    DB.get_or_init(|| Database::load_bundled().ok()).as_ref()
}

#[derive(Debug, Clone, serde::Serialize, Default)]
pub struct ProfileLensEdits {
    pub k1: f64,
    pub k2: f64,
    pub k3: f64,
    pub vk1: f64,
    pub vk2: f64,
    pub vk3: f64,
    pub ca_red_scale_x10000: f64,
    pub ca_blue_scale_x10000: f64,
}

#[derive(Debug, Clone, serde::Serialize, Default)]
pub struct LensProfileMatch {
    pub matched: bool,
    pub lens: Option<String>,
    pub focal_length: Option<f32>,
    pub aperture: Option<f32>,
    pub edits: Option<ProfileLensEdits>,
}

fn normalize_aperture(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let bytes = name.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if (c == 'F' || c == 'f')
            && i + 1 < bytes.len()
            && (bytes[i + 1].is_ascii_digit() || bytes[i + 1] as char == '/')
        {
            out.push_str("f/");
            if bytes[i + 1] as char == '/' {
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

fn strip_prefix(name: &str) -> Option<String> {
    for p in [
        "FE ", "EF ", "EF-S ", "RF ", "Z ", "DG ", "DC ", "AF ", "AF-S ",
    ] {
        if let Some(rest) = name.strip_prefix(p) {
            return Some(rest.to_string());
        }
    }
    None
}

fn lookup_lens(
    db: &Database,
    camera: Option<&lensfun::Camera>,
    make: Option<&str>,
    lens_name: &str,
) -> Option<Lens> {
    let mut candidates: Vec<String> = Vec::new();
    let push = |c: &mut Vec<String>, s: String| {
        if !c.contains(&s) {
            c.push(s);
        }
    };
    push(&mut candidates, lens_name.to_string());
    push(&mut candidates, normalize_aperture(lens_name));
    let make_stripped: Option<String> = make.and_then(|m| {
        let needle = format!("{m} ");
        lens_name
            .to_lowercase()
            .starts_with(&needle.to_lowercase())
            .then(|| lens_name[needle.len()..].to_string())
    });
    if let Some(ms) = make_stripped.as_deref() {
        push(&mut candidates, ms.to_string());
        push(&mut candidates, normalize_aperture(ms));
        if let Some(s) = strip_prefix(ms) {
            push(&mut candidates, normalize_aperture(&s));
            push(&mut candidates, s);
        }
    }
    if let Some(s) = strip_prefix(lens_name) {
        push(&mut candidates, normalize_aperture(&s));
        push(&mut candidates, s);
    }
    if let Some(m) = make {
        push(&mut candidates, format!("{m} {lens_name}"));
        push(
            &mut candidates,
            normalize_aperture(&format!("{m} {lens_name}")),
        );
    }
    for q in &candidates {
        let found = db.find_lenses(camera, q);
        if let Some(l) = found.into_iter().next() {
            return Some(l.clone());
        }
        if camera.is_some() {
            let found2 = db.find_lenses(None, q);
            if let Some(l) = found2.into_iter().next() {
                return Some(l.clone());
            }
        }
    }
    None
}

pub fn lookup(exif: &ExifInfo) -> LensProfileMatch {
    let Some(db) = db() else {
        return LensProfileMatch::default();
    };
    let make = exif.make.as_deref();
    let model = exif.model.as_deref();
    let cameras = match model {
        Some(m) => db.find_cameras(make, m),
        None => Vec::new(),
    };
    let camera = cameras.first().copied();
    let lens_name = exif.lens_model.as_deref().unwrap_or_default();
    if lens_name.is_empty() {
        tracing::warn!(?make, ?model, "lens_profile: exif.lens_model missing");
        return LensProfileMatch::default();
    }
    let Some(lens) = lookup_lens(db, camera, make, lens_name) else {
        tracing::warn!(?make, ?model, lens_name, "lens_profile: no lens match");
        return LensProfileMatch::default();
    };
    let focal = exif
        .focal_length
        .map(|f| f as f32)
        .unwrap_or(lens.focal_min);
    let aperture = exif.f_number.map(|f| f as f32).unwrap_or(lens.aperture_min);
    let mut edits = ProfileLensEdits::default();

    if let Some(cd) = lens.interpolate_distortion(focal)
        && let Some((k1, k2, k3)) = fit_distortion(&cd)
    {
        edits.k1 = k1 as f64;
        edits.k2 = k2 as f64;
        edits.k3 = k3 as f64;
    }
    if let Some(ct) = lens.interpolate_tca(focal)
        && let Some((red, blue)) = fit_tca(&ct)
    {
        edits.ca_red_scale_x10000 = ((red - 1.0) as f64) * 10000.0;
        edits.ca_blue_scale_x10000 = ((blue - 1.0) as f64) * 10000.0;
    }
    if let Some(cv) = lens.interpolate_vignetting(focal, aperture, 1000.0)
        && let Some((vk1, vk2, vk3)) = fit_vignetting(&cv)
    {
        edits.vk1 = vk1 as f64;
        edits.vk2 = vk2 as f64;
        edits.vk3 = vk3 as f64;
    }

    let has_any = edits.k1 != 0.0
        || edits.k2 != 0.0
        || edits.k3 != 0.0
        || edits.vk1 != 0.0
        || edits.vk2 != 0.0
        || edits.vk3 != 0.0
        || edits.ca_red_scale_x10000 != 0.0
        || edits.ca_blue_scale_x10000 != 0.0;

    LensProfileMatch {
        matched: true,
        lens: Some(lens.model.clone()),
        focal_length: Some(focal),
        aperture: Some(aperture),
        edits: has_any.then_some(edits),
    }
}

const FIT_RADII: [f32; 10] = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];

fn distortion_s_target(model: &DistortionModel, r: f32) -> f32 {
    match *model {
        DistortionModel::None => 1.0,
        DistortionModel::Poly3 { k1 } => (1.0 - k1) + k1 * r * r,
        DistortionModel::Poly5 { k1, k2 } => 1.0 + k1 * r * r + k2 * r * r * r * r,
        DistortionModel::Ptlens { a, b, c } => {
            a * r * r * r + b * r * r + c * r + (1.0 - a - b - c)
        }
    }
}

fn fit_distortion(cd: &CalibDistortion) -> Option<(f32, f32, f32)> {
    if matches!(cd.model, DistortionModel::None) {
        return None;
    }
    let mut at = [[0.0f64; 3]; 3];
    let mut bt = [0.0f64; 3];
    for &r in &FIT_RADII {
        let r2 = (r * r) as f64;
        let r4 = r2 * r2;
        let r6 = r4 * r2;
        let phi = [r2, r4, r6];
        let t = (distortion_s_target(&cd.model, r) - 1.0) as f64;
        for j in 0..3 {
            for k in 0..3 {
                at[j][k] += phi[j] * phi[k];
            }
            bt[j] += phi[j] * t;
        }
    }
    let sol = solve_3x3(at, bt)?;
    Some((sol[0] as f32, sol[1] as f32, sol[2] as f32))
}

fn fit_tca(ct: &CalibTca) -> Option<(f32, f32)> {
    match ct.model {
        TcaModel::None => None,
        TcaModel::Linear { kr, kb } => Some((kr, kb)),
        TcaModel::Poly3 { red, blue } => {
            let r = 0.7f32;
            let red_scale = red[0] + red[1] * r + red[2] * r * r;
            let blue_scale = blue[0] + blue[1] * r + blue[2] * r * r;
            Some((red_scale, blue_scale))
        }
    }
}

fn fit_vignetting(cv: &CalibVignetting) -> Option<(f32, f32, f32)> {
    match cv.model {
        VignettingModel::None => None,
        VignettingModel::Pa { k1, k2, k3 } => Some((k1, k2, k3)),
    }
}

fn solve_3x3(a: [[f64; 3]; 3], b: [f64; 3]) -> Option<[f64; 3]> {
    let det = a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1])
        - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0])
        + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0]);
    if det.abs() < 1e-18 {
        return None;
    }
    let col = |k: usize| {
        let mut m = a;
        for row in 0..3 {
            m[row][k] = b[row];
        }
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    };
    Some([col(0) / det, col(1) / det, col(2) / det])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_poly5_is_exact() {
        let cd = CalibDistortion {
            focal: 24.0,
            model: DistortionModel::Poly5 {
                k1: 0.02,
                k2: -0.005,
            },
            real_focal: None,
        };
        let (k1, k2, k3) = fit_distortion(&cd).unwrap();
        if (k1 - 0.02).abs() > 1e-3 {
            panic!("k1 {k1}");
        }
        if (k2 - (-0.005)).abs() > 1e-3 {
            panic!("k2 {k2}");
        }
        if k3.abs() > 1e-3 {
            panic!("k3 {k3}");
        }
    }

    #[test]
    fn fit_tca_linear_direct() {
        let ct = CalibTca {
            focal: 24.0,
            model: TcaModel::Linear {
                kr: 1.002,
                kb: 0.998,
            },
        };
        let (red, blue) = fit_tca(&ct).unwrap();
        if (red - 1.002).abs() > 1e-6 {
            panic!("red {red}");
        }
        if (blue - 0.998).abs() > 1e-6 {
            panic!("blue {blue}");
        }
    }

    #[test]
    fn fit_vignetting_pa_passthrough() {
        let cv = CalibVignetting {
            focal: 24.0,
            aperture: 2.8,
            distance: 1000.0,
            model: VignettingModel::Pa {
                k1: -0.3,
                k2: 0.05,
                k3: 0.0,
            },
        };
        let (k1, k2, k3) = fit_vignetting(&cv).unwrap();
        if (k1 - (-0.3)).abs() > 1e-6 || (k2 - 0.05).abs() > 1e-6 || k3.abs() > 1e-6 {
            panic!("vk mismatch: {k1} {k2} {k3}");
        }
    }

    #[test]
    fn normalize_aperture_inserts_slash() {
        if normalize_aperture("FE 35mm F1.8") != "FE 35mm f/1.8" {
            panic!("got {}", normalize_aperture("FE 35mm F1.8"));
        }
        if normalize_aperture("Sony FE 35mm f/1.8") != "Sony FE 35mm f/1.8" {
            panic!("idempotent failure");
        }
    }

    #[test]
    fn strip_prefix_drops_fe() {
        if strip_prefix("FE 35mm F1.8").as_deref() != Some("35mm F1.8") {
            panic!("strip_prefix failed");
        }
    }

    #[test]
    fn lookup_finds_sony_fe_lens() {
        let Some(db) = db() else {
            return;
        };
        let lens = lookup_lens(db, None, Some("SONY"), "Sony FE 35mm F1.8");
        let Some(l) = lens else {
            panic!("no lens found for 'Sony FE 35mm F1.8'");
        };
        if !l.model.contains("FE 35mm") {
            panic!("got wrong lens: {}", l.model);
        }
    }
}
