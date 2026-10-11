use super::dcp::dcp_illuminant_cct;
use super::matrix::{
    D50_XY, bradford_cat, cam_to_srgb_matrix, identity_3x3, inverse_3x3, is_unusable_matrix,
    mat3_mul, mat3_vec,
};
use super::temperature::Temperature;
use super::{D65_XY, SRGB_TO_XYZ_D65, XYZ_TO_SRGB_D65};
use crate::dcp::{DcpIlluminant, DcpProfile};
use crate::edits::WhiteBalance;

const NEUTRAL_PASSES: usize = 30;
const NEUTRAL_FLOOR: f32 = 0.001;

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

    pub fn as_shot(&self) -> Temperature {
        let xy = match self {
            SceneWhite::Display => [D65_XY.0, D65_XY.1],
            SceneWhite::Camera(camera) => camera.as_shot,
        };
        Temperature::from_xy(xy.map(f64::from))
    }

    pub fn user_matrix(&self, wb: &WhiteBalance) -> [[f32; 3]; 3] {
        match self {
            SceneWhite::Display => {
                let [x, y] = target([D65_XY.0, D65_XY.1], wb);
                let cat = bradford_cat((x, y), D65_XY);
                mat3_mul(&XYZ_TO_SRGB_D65, &mat3_mul(&cat, &SRGB_TO_XYZ_D65))
            }
            SceneWhite::Camera(camera) => camera.user_matrix(wb),
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

    pub fn weight_at(&self, wb: &WhiteBalance) -> f32 {
        self.weight(target(self.as_shot, wb))
    }

    pub fn user_matrix(&self, wb: &WhiteBalance) -> [[f32; 3]; 3] {
        let target = target(self.as_shot, wb);
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
        let cct = Temperature::from_xy(xy.map(f64::from)).kelvin as f32;
        let inv = 1.0 / cct.clamp(cct1.min(cct2), cct1.max(cct2));
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
        let peak = n[0].max(n[1]).max(n[2]);
        let pinned = n.map(|c| (c / peak).clamp(NEUTRAL_FLOOR, 1.0));
        pinned.map(|c| c / pinned[1])
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

fn target(as_shot: [f32; 2], wb: &WhiteBalance) -> [f32; 2] {
    if wb.is_as_shot() {
        return as_shot;
    }
    wb.resolve(Temperature::from_xy(as_shot.map(f64::from)))
        .xy()
        .map(|c| c as f32)
}

#[cfg(test)]
mod tests;
