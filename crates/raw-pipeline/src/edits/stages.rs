use super::{DetailEdits, Edits, GeometryEdits, LensBlurEdits};

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
            capture_corner_boost: self.detail.capture_corner_boost,
            ai_denoise: self.detail.ai_denoise,
            ai_denoise_strength: self.detail.ai_denoise_strength,
            ai_denoise_detail: self.detail.ai_denoise_detail,
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
            lens_blur: LensBlurEdits {
                depth_raster_id: self.lens_blur.depth_raster_id.clone(),
                ..LensBlurEdits::default()
            },
            ..Self::default()
        };
        out.color.dcp = self.color.dcp.clone();
        out
    }
}
