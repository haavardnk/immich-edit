use std::sync::Arc;

use raw_pipeline::ai_denoise::DenoisePlane;
use raw_pipeline::frame::RawFrame;

pub fn denoise_plane(frame: &RawFrame, key: u64, rgb: &[f32]) -> Arc<DenoisePlane> {
    Arc::new(DenoisePlane::from_rgb(key, frame.meta.width, frame.meta.height, rgb).unwrap())
}
