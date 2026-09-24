use super::{DetailEdits, Edits, GeometryEdits};

impl Edits {
    pub fn sensor_stage(&self) -> Self {
        let detail = DetailEdits {
            luma_nr_amount: self.detail.luma_nr_amount,
            luma_nr_detail: self.detail.luma_nr_detail,
            luma_nr_contrast: self.detail.luma_nr_contrast,
            color_nr_amount: self.detail.color_nr_amount,
            color_nr_detail: self.detail.color_nr_detail,
            color_nr_smoothness: self.detail.color_nr_smoothness,
            capture_sharpen: self.detail.capture_sharpen,
            ..DetailEdits::default()
        };
        let geometry = GeometryEdits {
            rotate: self.geometry.rotate,
            rotate_angle: self.geometry.rotate_angle,
            crop: self.geometry.crop,
            ..GeometryEdits::default()
        };
        let mut out = Self {
            detail,
            lens: self.lens,
            geometry,
            retouch: self.retouch.clone(),
            ..Self::default()
        };
        out.color.dcp = self.color.dcp.clone();
        out
    }
}
