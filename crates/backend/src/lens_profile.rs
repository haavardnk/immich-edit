use crate::immich::dto::ExifInfo;
use lensfun::{
    CalibDistortion, CalibTca, CalibVignetting, Database, DistortionModel, Lens, TcaModel,
    VignettingModel,
};
use raw_pipeline::edits::LensEdits;
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
    let aperture_opt = exif.f_number.map(|f| f as f32);
    let edits = profile_edits(&lens, focal, aperture_opt, camera.map(|c| c.crop_factor));

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
        aperture: aperture_opt,
        edits: has_any.then_some(edits),
    }
}

fn profile_edits(
    lens: &Lens,
    focal: f32,
    aperture: Option<f32>,
    camera_crop: Option<f32>,
) -> ProfileLensEdits {
    let calib_crop = if lens.crop_factor > 0.0 {
        lens.crop_factor
    } else {
        1.0
    };
    let image_crop = camera_crop.filter(|c| *c > 0.0).unwrap_or(calib_crop);
    let crop_ratio = (calib_crop / image_crop) as f64;
    let corner = (crop_ratio * (lens.aspect_ratio as f64).hypot(1.0)) as f32;
    let mut edits = ProfileLensEdits::default();

    if let Some(cd) = lens.interpolate_distortion(focal)
        && let Some((k1, k2, k3)) = fit_distortion(&cd, corner)
    {
        edits.k1 = k1 as f64;
        edits.k2 = k2 as f64;
        edits.k3 = k3 as f64;
    }
    if let Some(ct) = lens.interpolate_tca(focal)
        && let Some((red, blue)) = fit_tca(&ct, corner)
    {
        edits.ca_red_scale_x10000 = ((red - 1.0) as f64) * 10000.0;
        edits.ca_blue_scale_x10000 = ((blue - 1.0) as f64) * 10000.0;
    }
    if let Some(aperture) = aperture
        && let Some(cv) = lens.interpolate_vignetting(focal, aperture, 1000.0)
        && let Some((vk1, vk2, vk3)) = fit_vignetting(&cv)
    {
        let r2 = crop_ratio * crop_ratio;
        let r4 = r2 * r2;
        let r6 = r4 * r2;
        edits.vk1 = (vk1 as f64) * r2;
        edits.vk2 = (vk2 as f64) * r4;
        edits.vk3 = (vk3 as f64) * r6;
    }
    edits
}

pub fn reproject_lens(lens: LensEdits, exif: Option<&ExifInfo>) -> LensEdits {
    let profile = exif.map(lookup).and_then(|m| m.edits).unwrap_or_default();
    LensEdits {
        k1: profile.k1,
        k2: profile.k2,
        k3: profile.k3,
        vk1: profile.vk1,
        vk2: profile.vk2,
        vk3: profile.vk3,
        ca_red_scale_x10000: profile.ca_red_scale_x10000,
        ca_blue_scale_x10000: profile.ca_blue_scale_x10000,
        ..lens
    }
}

pub fn apply_auto(lens: LensEdits, profile: Option<&ProfileLensEdits>) -> LensEdits {
    if lens.profile_enabled.is_some() {
        return lens;
    }
    let Some(profile) = profile else {
        return lens;
    };
    LensEdits {
        profile_enabled: Some(true),
        constrain_crop: true,
        k1: profile.k1,
        k2: profile.k2,
        k3: profile.k3,
        vk1: profile.vk1,
        vk2: profile.vk2,
        vk3: profile.vk3,
        ..lens
    }
}

fn fit_radii() -> impl Iterator<Item = f64> {
    (1..=20).map(|i| i as f64 / 20.0)
}

fn distortion_s_target(model: &DistortionModel, r: f64) -> f64 {
    match *model {
        DistortionModel::None => 1.0,
        DistortionModel::Poly3 { k1 } => {
            let d = 1.0 - k1 as f64;
            1.0 + k1 as f64 / d.powi(3) * r * r
        }
        DistortionModel::Poly5 { k1, k2 } => 1.0 + k1 as f64 * r * r + k2 as f64 * r.powi(4),
        DistortionModel::Ptlens { a, b, c } => {
            let d = 1.0 - a as f64 - b as f64 - c as f64;
            1.0 + a as f64 / d.powi(4) * r.powi(3)
                + b as f64 / d.powi(3) * r * r
                + c as f64 / d.powi(2) * r
        }
    }
}

fn fit_distortion(cd: &CalibDistortion, corner: f32) -> Option<(f32, f32, f32)> {
    if matches!(cd.model, DistortionModel::None) {
        return None;
    }
    let mut at = [[0.0f64; 3]; 3];
    let mut bt = [0.0f64; 3];
    for r in fit_radii() {
        let r2 = r * r;
        let r4 = r2 * r2;
        let r6 = r4 * r2;
        let phi = [r2, r4, r6];
        let t = distortion_s_target(&cd.model, r * corner as f64) - 1.0;
        for j in 0..3 {
            for k in 0..3 {
                at[j][k] += r2 * phi[j] * phi[k];
            }
            bt[j] += r2 * phi[j] * t;
        }
    }
    let sol = solve_3x3(at, bt)?;
    Some((sol[0] as f32, sol[1] as f32, sol[2] as f32))
}

fn fit_tca(ct: &CalibTca, corner: f32) -> Option<(f32, f32)> {
    match ct.model {
        TcaModel::None => None,
        TcaModel::Linear { kr, kb } => Some((kr, kb)),
        TcaModel::Poly3 { red, blue } => {
            let scale = |t: [f32; 3]| -> f32 {
                let mut num = 0.0f64;
                let mut den = 0.0f64;
                for r in fit_radii() {
                    let rh = r * corner as f64;
                    let w = r * r;
                    num += w * (t[0] as f64 + t[1] as f64 * rh + t[2] as f64 * rh * rh);
                    den += w;
                }
                (num / den) as f32
            };
            Some((scale(red), scale(blue)))
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
    use lensfun::Modifier;

    #[test]
    fn reproject_without_exif_zeroes_coeffs_keeps_flags() {
        let lens = LensEdits {
            profile_enabled: Some(true),
            ca_enabled: true,
            constrain_crop: true,
            distortion_amount: 80.0,
            vignette_amount: 60.0,
            k1: 0.1,
            k2: 0.2,
            k3: 0.3,
            vk1: 0.4,
            vk2: 0.5,
            vk3: 0.6,
            ca_red_scale_x10000: 7.0,
            ca_blue_scale_x10000: 8.0,
        };
        let out = reproject_lens(lens, None);
        assert_eq!(out.profile_enabled, Some(true));
        assert!(out.ca_enabled);
        assert!(out.constrain_crop);
        assert_eq!(out.distortion_amount, 80.0);
        assert_eq!(out.vignette_amount, 60.0);
        assert_eq!(out.k1, 0.0);
        assert_eq!(out.k2, 0.0);
        assert_eq!(out.k3, 0.0);
        assert_eq!(out.vk1, 0.0);
        assert_eq!(out.vk2, 0.0);
        assert_eq!(out.vk3, 0.0);
        assert_eq!(out.ca_red_scale_x10000, 0.0);
        assert_eq!(out.ca_blue_scale_x10000, 0.0);
    }

    #[test]
    fn apply_auto_only_fills_unset_profiles() {
        let profile = ProfileLensEdits {
            k1: -0.1,
            k2: 0.02,
            k3: 0.0,
            vk1: -0.3,
            vk2: 0.0,
            vk3: 0.0,
            ca_red_scale_x10000: 40.0,
            ca_blue_scale_x10000: -40.0,
        };
        let auto = apply_auto(LensEdits::default(), Some(&profile));
        assert_eq!(auto.profile_enabled, Some(true));
        assert!(auto.constrain_crop);
        assert_eq!(auto.k1, -0.1);
        assert_eq!(auto.vk1, -0.3);
        assert!(!auto.ca_enabled);
        assert_eq!(auto.ca_red_scale_x10000, 0.0);

        for explicit in [Some(true), Some(false)] {
            let lens = LensEdits {
                profile_enabled: explicit,
                ..Default::default()
            };
            let out = apply_auto(lens, Some(&profile));
            assert_eq!(out.profile_enabled, explicit);
            assert_eq!(out.k1, 0.0);
        }

        let unmatched = apply_auto(LensEdits::default(), None);
        assert_eq!(unmatched.profile_enabled, None);
    }

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
        let (k1, k2, k3) = fit_distortion(&cd, 1.0).unwrap();
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
        let (red, blue) = fit_tca(&ct, 1.8).unwrap();
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

    const W: u32 = 6000;
    const H: u32 = 4000;
    const PROBES: [(f32, f32); 4] = [(1.0, 1.0), (1.0, 0.0), (0.0, 1.0), (0.5, 0.5)];

    fn synthetic_lens(distortion: DistortionModel, tca: TcaModel) -> Lens {
        Lens {
            model: "synthetic".to_string(),
            focal_min: 24.0,
            focal_max: 24.0,
            crop_factor: 1.0,
            aspect_ratio: 1.5,
            calib_distortion: vec![CalibDistortion {
                focal: 24.0,
                model: distortion,
                real_focal: None,
            }],
            calib_tca: vec![CalibTca {
                focal: 24.0,
                model: tca,
            }],
            ..Default::default()
        }
    }

    fn probe_offsets() -> impl Iterator<Item = (f32, f32, f32)> {
        let half_diag = 0.5 * (W as f32).hypot(H as f32);
        PROBES.into_iter().map(move |(fx, fy)| {
            let dx = fx * (W - 1) as f32 * 0.5;
            let dy = fy * (H - 1) as f32 * 0.5;
            (dx, dy, dx.hypot(dy) / half_diag)
        })
    }

    #[test]
    fn distortion_matches_lensfun_modifier() {
        let cx = (W - 1) as f32 * 0.5;
        let cy = (H - 1) as f32 * 0.5;
        for (model, camera_crop, limit_px) in [
            (DistortionModel::Poly3 { k1: -0.03 }, 1.0, 0.5),
            (DistortionModel::Poly3 { k1: -0.03 }, 1.5, 0.5),
            (
                DistortionModel::Poly5 {
                    k1: -0.05,
                    k2: 0.01,
                },
                1.0,
                0.5,
            ),
            (
                DistortionModel::Ptlens {
                    a: 0.01,
                    b: -0.03,
                    c: 0.02,
                },
                1.0,
                3.0,
            ),
        ] {
            let lens = synthetic_lens(model, TcaModel::None);
            let edits = profile_edits(&lens, 24.0, None, Some(camera_crop));
            let mut modifier = Modifier::new(&lens, 24.0, camera_crop, W, H, false);
            if !modifier.enable_distortion_correction(&lens) {
                panic!("{model:?}: lensfun rejected the profile");
            }
            for (dx, dy, r) in probe_offsets() {
                let r2 = (r * r) as f64;
                let s = (1.0 + edits.k1 * r2 + edits.k2 * r2 * r2 + edits.k3 * r2 * r2 * r2) as f32;
                let mut lensfun = [0.0f32; 2];
                modifier.apply_geometry_distortion(cx + dx, cy + dy, 1, 1, &mut lensfun);
                let err = (lensfun[0] - cx - dx * s).hypot(lensfun[1] - cy - dy * s);
                if err > limit_px {
                    panic!("{model:?} crop {camera_crop} at ({dx}, {dy}): {err} px off lensfun");
                }
            }
        }
    }

    #[test]
    fn tca_matches_lensfun_modifier() {
        let cx = (W - 1) as f32 * 0.5;
        let cy = (H - 1) as f32 * 0.5;
        for (model, limit_px) in [
            (
                TcaModel::Linear {
                    kr: 1.0004,
                    kb: 0.9996,
                },
                0.05,
            ),
            (
                TcaModel::Poly3 {
                    red: [1.0002, 0.0, 0.0002],
                    blue: [0.9998, 0.0001, -0.0002],
                },
                1.0,
            ),
        ] {
            let lens = synthetic_lens(DistortionModel::None, model);
            let edits = profile_edits(&lens, 24.0, None, None);
            let red = 1.0 + edits.ca_red_scale_x10000 as f32 / 10000.0;
            let blue = 1.0 + edits.ca_blue_scale_x10000 as f32 / 10000.0;
            let mut modifier = Modifier::new(&lens, 24.0, 1.0, W, H, false);
            if !modifier.enable_tca_correction(&lens) {
                panic!("{model:?}: lensfun rejected the profile");
            }
            for (dx, dy, _) in probe_offsets() {
                let mut lensfun = [0.0f32; 6];
                modifier.apply_subpixel_distortion(cx + dx, cy + dy, 1, 1, &mut lensfun);
                let red_err = (lensfun[0] - cx - dx * red).hypot(lensfun[1] - cy - dy * red);
                let blue_err = (lensfun[4] - cx - dx * blue).hypot(lensfun[5] - cy - dy * blue);
                if red_err.max(blue_err) > limit_px {
                    panic!(
                        "{model:?} at ({dx}, {dy}): red {red_err} px, blue {blue_err} px off lensfun"
                    );
                }
            }
        }
    }
}
