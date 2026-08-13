use std::hash::{Hash, Hasher};

use super::GpuRenderer;
use crate::edits::Edits;
use crate::frame::RawFrame;

pub(super) fn atmosphere_cache_key(frame: &RawFrame, edits: &Edits, dims: (u32, u32)) -> u64 {
    let mut e = edits.clone();
    e.basic.dehaze = 0.0;
    let json = serde_json::to_vec(&e).unwrap_or_default();
    let mut h = std::collections::hash_map::DefaultHasher::new();
    GpuRenderer::frame_key(frame).hash(&mut h);
    dims.0.hash(&mut h);
    dims.1.hash(&mut h);
    json.hash(&mut h);
    h.finish()
}

pub(super) fn wb_cache_key(
    frame: &RawFrame,
    edits: &Edits,
    dims: (u32, u32),
    cam_to_srgb: [[f32; 3]; 3],
) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    GpuRenderer::frame_key(frame).hash(&mut h);
    dims.0.hash(&mut h);
    dims.1.hash(&mut h);
    edits.basic.wb_temp.to_bits().hash(&mut h);
    edits.basic.wb_tint.to_bits().hash(&mut h);
    for row in cam_to_srgb {
        for v in row {
            v.to_bits().hash(&mut h);
        }
    }
    let lens_json = serde_json::to_vec(&edits.lens).unwrap_or_default();
    lens_json.hash(&mut h);
    h.finish()
}

pub(super) fn nr_cache_key(
    frame: &RawFrame,
    edits: &Edits,
    dims: (u32, u32),
    cam_to_srgb: [[f32; 3]; 3],
) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    wb_cache_key(frame, edits, dims, cam_to_srgb).hash(&mut h);
    let retouch_json = serde_json::to_vec(&edits.retouch).unwrap_or_default();
    retouch_json.hash(&mut h);
    edits.detail.hash_nr(&mut h);
    h.finish()
}

pub(super) fn capture_cache_key(
    frame: &RawFrame,
    edits: &Edits,
    dims: (u32, u32),
    cam_to_srgb: [[f32; 3]; 3],
    sigma: f32,
) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    nr_cache_key(frame, edits, dims, cam_to_srgb).hash(&mut h);
    sigma.to_bits().hash(&mut h);
    h.finish()
}
