use serde::{Deserialize, Serialize};

pub const MAX_LENS_COEFF: f64 = 2.0;

fn lens_amount_default() -> f64 {
    100.0
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct LensEdits {
    #[serde(default)]
    pub profile_enabled: Option<bool>,
    #[serde(default)]
    pub ca_enabled: bool,
    #[serde(default)]
    pub constrain_crop: bool,
    #[serde(default = "lens_amount_default")]
    pub distortion_amount: f64,
    #[serde(default = "lens_amount_default")]
    pub vignette_amount: f64,
    #[serde(default)]
    pub k1: f64,
    #[serde(default)]
    pub k2: f64,
    #[serde(default)]
    pub k3: f64,
    #[serde(default)]
    pub vk1: f64,
    #[serde(default)]
    pub vk2: f64,
    #[serde(default)]
    pub vk3: f64,
    #[serde(default)]
    pub ca_red_scale_x10000: f64,
    #[serde(default)]
    pub ca_blue_scale_x10000: f64,
}

impl Default for LensEdits {
    fn default() -> Self {
        Self {
            profile_enabled: None,
            ca_enabled: false,
            constrain_crop: false,
            distortion_amount: 100.0,
            vignette_amount: 100.0,
            k1: 0.0,
            k2: 0.0,
            k3: 0.0,
            vk1: 0.0,
            vk2: 0.0,
            vk3: 0.0,
            ca_red_scale_x10000: 0.0,
            ca_blue_scale_x10000: 0.0,
        }
    }
}

impl LensEdits {
    pub fn profile_active(&self) -> bool {
        self.profile_enabled.unwrap_or(false)
    }
    pub fn distortion_active(&self) -> bool {
        self.profile_active()
            && self.distortion_amount != 0.0
            && (self.k1 != 0.0 || self.k2 != 0.0 || self.k3 != 0.0)
    }
    pub fn vignette_active(&self) -> bool {
        self.profile_active()
            && self.vignette_amount != 0.0
            && (self.vk1 != 0.0 || self.vk2 != 0.0 || self.vk3 != 0.0)
    }
    pub fn ca_active(&self) -> bool {
        self.ca_enabled && (self.ca_red_scale_x10000 != 0.0 || self.ca_blue_scale_x10000 != 0.0)
    }
    pub fn any_active(&self) -> bool {
        self.distortion_active() || self.vignette_active() || self.ca_active()
    }
    pub fn effective_k(&self) -> (f64, f64, f64) {
        let s = self.distortion_amount / 100.0;
        (self.k1 * s, self.k2 * s, self.k3 * s)
    }
    pub fn ca_scales(&self) -> (f64, f64) {
        (
            1.0 + self.ca_red_scale_x10000 / 10000.0,
            1.0 + self.ca_blue_scale_x10000 / 10000.0,
        )
    }
    pub fn clamped(&self) -> Self {
        Self {
            profile_enabled: self.profile_enabled,
            ca_enabled: self.ca_enabled,
            constrain_crop: self.constrain_crop,
            distortion_amount: self.distortion_amount.clamp(0.0, 200.0),
            vignette_amount: self.vignette_amount.clamp(0.0, 200.0),
            k1: self.k1.clamp(-MAX_LENS_COEFF, MAX_LENS_COEFF),
            k2: self.k2.clamp(-MAX_LENS_COEFF, MAX_LENS_COEFF),
            k3: self.k3.clamp(-MAX_LENS_COEFF, MAX_LENS_COEFF),
            vk1: self.vk1.clamp(-MAX_LENS_COEFF, MAX_LENS_COEFF),
            vk2: self.vk2.clamp(-MAX_LENS_COEFF, MAX_LENS_COEFF),
            vk3: self.vk3.clamp(-MAX_LENS_COEFF, MAX_LENS_COEFF),
            ca_red_scale_x10000: self.ca_red_scale_x10000.clamp(-500.0, 500.0),
            ca_blue_scale_x10000: self.ca_blue_scale_x10000.clamp(-500.0, 500.0),
        }
    }
}
