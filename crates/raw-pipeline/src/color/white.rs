use super::dcp::dcp_illuminant_cct;
use super::matrix::{
    D50_XY, bradford_cat, cam_to_srgb_matrix, identity_3x3, inverse_3x3, is_unusable_matrix,
    mat3_mul, mat3_vec,
};
use super::{D65_XY, SRGB_TO_XYZ_D65, XYZ_TO_SRGB_D65};
use crate::dcp::{DcpIlluminant, DcpProfile};

pub const WB_TEMP_STOPS: f32 = 1.5;
pub const WB_TINT_DUV: f32 = 0.0003;
const MIRED_MIN: f32 = 40.0;
const MIRED_MAX: f32 = 600.0;
const NEUTRAL_PASSES: usize = 30;

#[derive(Clone, Copy, Debug)]
pub struct Calibration {
    pub cct: f32,
    pub xyz_to_cam: [[f32; 3]; 3],
    pub forward: Option<[[f32; 3]; 3]>,
}

#[derive(Clone, Debug)]
pub struct CameraWhite {
    first: Calibration,
    second: Option<Calibration>,
    fixed_weight: Option<f32>,
    as_shot: [f32; 2],
}

#[derive(Clone, Debug)]
pub enum SceneWhite {
    Display,
    Camera(CameraWhite),
}

impl SceneWhite {
    pub fn cam_to_srgb(&self) -> [[f32; 3]; 3] {
        match self {
            SceneWhite::Display => identity_3x3(),
            SceneWhite::Camera(camera) => camera.cam_to_srgb(),
        }
    }

    pub fn user_matrix(&self, temp: f64, tint: f64) -> [[f32; 3]; 3] {
        match self {
            SceneWhite::Display => {
                let [x, y] = shifted([D65_XY.0, D65_XY.1], temp as f32, tint as f32);
                let cat = bradford_cat((x, y), D65_XY);
                mat3_mul(&XYZ_TO_SRGB_D65, &mat3_mul(&cat, &SRGB_TO_XYZ_D65))
            }
            SceneWhite::Camera(camera) => camera.user_matrix(temp, tint),
        }
    }
}

impl CameraWhite {
    pub fn new(
        first: Calibration,
        second: Option<Calibration>,
        fixed_weight: Option<f32>,
        wb_coeffs: [f32; 4],
    ) -> Self {
        let mut camera = Self {
            first,
            second,
            fixed_weight,
            as_shot: [D50_XY.0, D50_XY.1],
        };
        let neutral = [0, 1, 2].map(|c| 1.0 / wb_coeffs[c].max(1e-6));
        camera.as_shot = camera.neutral_to_xy(neutral);
        camera
    }

    pub fn from_dcp(profile: &DcpProfile, illuminant: DcpIlluminant, wb_coeffs: [f32; 4]) -> Self {
        let first = Calibration {
            cct: dcp_illuminant_cct(profile.calibration_illuminant1),
            xyz_to_cam: profile.color_matrix1,
            forward: profile.forward_matrix1,
        };
        let second = profile
            .calibration_illuminant2
            .zip(profile.color_matrix2)
            .map(|(code, xyz_to_cam)| Calibration {
                cct: dcp_illuminant_cct(code),
                xyz_to_cam,
                forward: profile.forward_matrix2,
            });
        let fixed_weight = match illuminant {
            DcpIlluminant::First => Some(1.0),
            DcpIlluminant::Second if second.is_some() => Some(0.0),
            DcpIlluminant::Second => Some(1.0),
            DcpIlluminant::Interpolated => None,
        };
        Self::new(first, second, fixed_weight, wb_coeffs)
    }

    pub fn from_matrices(
        matrices: &[(f32, [[f32; 3]; 4])],
        fallback: [[f32; 3]; 4],
        wb_coeffs: [f32; 4],
    ) -> Option<Self> {
        let (first, second) = match matrices {
            [first, .., last] => (*first, Some(*last)),
            _ => ((6504.0, fallback), None),
        };
        if is_unusable_matrix(&first.1) || second.is_some_and(|s| is_unusable_matrix(&s.1)) {
            return None;
        }
        let calibration = |(cct, m): (f32, [[f32; 3]; 4])| Calibration {
            cct,
            xyz_to_cam: [m[0], m[1], m[2]],
            forward: None,
        };
        Some(Self::new(
            calibration(first),
            second.map(calibration),
            None,
            wb_coeffs,
        ))
    }

    pub fn cam_to_srgb(&self) -> [[f32; 3]; 3] {
        self.cam_to_srgb_at(self.weight(self.as_shot))
    }

    pub fn weight_at(&self, temp: f64, tint: f64) -> f32 {
        self.weight(shifted(self.as_shot, temp as f32, tint as f32))
    }

    pub fn user_matrix(&self, temp: f64, tint: f64) -> [[f32; 3]; 3] {
        let target = shifted(self.as_shot, temp as f32, tint as f32);
        let n0 = self.neutral(self.as_shot);
        let n1 = self.neutral(target);
        let gains = [
            [n0[0] / n1[0], 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, n0[2] / n1[2]],
        ];
        let back = inverse_3x3(self.cam_to_srgb()).unwrap_or_else(identity_3x3);
        let forward = self.cam_to_srgb_at(self.weight(target));
        mat3_mul(&forward, &mat3_mul(&gains, &back))
    }

    fn weight(&self, xy: [f32; 2]) -> f32 {
        if let Some(weight) = self.fixed_weight {
            return weight;
        }
        let Some(second) = self.second else {
            return 1.0;
        };
        let (cct1, cct2) = (self.first.cct, second.cct);
        if (cct1 - cct2).abs() < 1.0 {
            return 1.0;
        }
        let inv = 1.0 / xy_to_cct(xy).clamp(cct1.min(cct2), cct1.max(cct2));
        ((inv - 1.0 / cct2) / (1.0 / cct1 - 1.0 / cct2)).clamp(0.0, 1.0)
    }

    fn xyz_to_cam(&self, weight: f32) -> [[f32; 3]; 3] {
        match self.second {
            Some(second) => lerp3(&self.first.xyz_to_cam, &second.xyz_to_cam, weight),
            None => self.first.xyz_to_cam,
        }
    }

    fn cam_to_srgb_at(&self, weight: f32) -> [[f32; 3]; 3] {
        let second = self.second.and_then(|s| s.forward);
        let forward = match (self.first.forward, second) {
            (Some(a), Some(b)) => Some(lerp3(&a, &b, weight)),
            (a, _) => a,
        };
        let Some(forward) = forward else {
            let m = self.xyz_to_cam(weight);
            return cam_to_srgb_matrix([m[0], m[1], m[2], [0.0; 3]]);
        };
        let cat = bradford_cat(D50_XY, D65_XY);
        mat3_mul(&XYZ_TO_SRGB_D65, &mat3_mul(&cat, &forward))
    }

    fn neutral(&self, xy: [f32; 2]) -> [f32; 3] {
        let n = mat3_vec(&self.xyz_to_cam(self.weight(xy)), xy_to_xyz(xy));
        n.map(|c| c / n[1])
    }

    fn neutral_to_xy(&self, neutral: [f32; 3]) -> [f32; 2] {
        let mut xy = [D50_XY.0, D50_XY.1];
        for _ in 0..NEUTRAL_PASSES {
            let Some(cam_to_xyz) = inverse_3x3(self.xyz_to_cam(self.weight(xy))) else {
                return [D65_XY.0, D65_XY.1];
            };
            let xyz = mat3_vec(&cam_to_xyz, neutral);
            let sum = xyz[0] + xyz[1] + xyz[2];
            if sum.is_nan() || sum <= 1e-6 {
                return [D65_XY.0, D65_XY.1];
            }
            let next = [xyz[0] / sum, xyz[1] / sum];
            let converged = (next[0] - xy[0]).abs() + (next[1] - xy[1]).abs() < 1e-7;
            xy = next;
            if converged {
                break;
            }
        }
        xy
    }
}

fn lerp3(a: &[[f32; 3]; 3], b: &[[f32; 3]; 3], weight: f32) -> [[f32; 3]; 3] {
    [0, 1, 2].map(|i| [0, 1, 2].map(|j| a[i][j] * weight + b[i][j] * (1.0 - weight)))
}

fn xy_to_xyz([x, y]: [f32; 2]) -> [f32; 3] {
    [x / y, 1.0, (1.0 - x - y) / y]
}

fn xy_to_cct([x, y]: [f32; 2]) -> f32 {
    let n = (x - 0.3320) / (0.1858 - y);
    449.0 * n * n * n + 3525.0 * n * n + 6823.3 * n + 5520.33
}

fn xy_to_uv([x, y]: [f32; 2]) -> [f32; 2] {
    let d = -2.0 * x + 12.0 * y + 3.0;
    [4.0 * x / d, 6.0 * y / d]
}

fn uv_to_xy([u, v]: [f32; 2]) -> [f32; 2] {
    let d = 2.0 * u - 8.0 * v + 4.0;
    [3.0 * u / d, 2.0 * v / d]
}

fn planck_uv(mired: f32) -> [f32; 2] {
    let t = 1.0e6 / mired;
    let t2 = t * t;
    let t3 = t2 * t;
    let x = if t <= 4000.0 {
        -0.266_123_9e9 / t3 - 0.234_358_9e6 / t2 + 0.877_695_6e3 / t + 0.179_910
    } else {
        -3.025_846_9e9 / t3 + 2.107_038e6 / t2 + 0.222_634_7e3 / t + 0.240_390
    };
    let y = if t <= 2222.0 {
        -1.106_381_4 * x * x * x - 1.348_110_2 * x * x + 2.185_558_3 * x - 0.202_196_83
    } else if t <= 4000.0 {
        -0.954_947_6 * x * x * x - 1.374_185_9 * x * x + 2.091_37 * x - 0.167_488_67
    } else {
        3.081_758 * x * x * x - 5.873_387 * x * x + 3.751_13 * x - 0.370_014_83
    };
    xy_to_uv([x, y])
}

fn locus_frame(mired: f32) -> ([f32; 2], [f32; 2], [f32; 2]) {
    let a = planck_uv(mired - 1.0);
    let b = planck_uv(mired + 1.0);
    let len = (b[0] - a[0]).hypot(b[1] - a[1]);
    let tangent = [(b[0] - a[0]) / len, (b[1] - a[1]) / len];
    let normal = if tangent[0] >= 0.0 {
        [-tangent[1], tangent[0]]
    } else {
        [tangent[1], -tangent[0]]
    };
    (planck_uv(mired), tangent, normal)
}

fn shifted(xy: [f32; 2], temp: f32, tint: f32) -> [f32; 2] {
    let mired = (1.0e6 / xy_to_cct(xy)).clamp(MIRED_MIN, MIRED_MAX);
    let target = (mired * (-temp * WB_TEMP_STOPS / 100.0).exp2()).clamp(MIRED_MIN, MIRED_MAX);
    let (p0, t0, n0) = locus_frame(mired);
    let (p1, t1, n1) = locus_frame(target);
    let uv = xy_to_uv(xy);
    let r = [uv[0] - p0[0], uv[1] - p0[1]];
    let along = r[0] * t0[0] + r[1] * t0[1];
    let across = r[0] * n0[0] + r[1] * n0[1] - tint * WB_TINT_DUV;
    uv_to_xy([
        p1[0] + along * t1[0] + across * n1[0],
        p1[1] + along * t1[1] + across * n1[1],
    ])
}

#[cfg(test)]
mod tests;
