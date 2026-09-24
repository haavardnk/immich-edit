use std::hash::{Hash, Hasher};

use crate::edits::{Edits, hash_strokes};
use crate::frame::RawFrame;

pub(super) struct StageKeys {
    pub wb: u64,
    pub nr: u64,
}

impl StageKeys {
    pub fn new(
        frame: &RawFrame,
        edits: &Edits,
        dims: (u32, u32),
        cam_to_srgb: [[f32; 3]; 3],
    ) -> Self {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        frame.cache_key().hash(&mut h);
        dims.0.hash(&mut h);
        dims.1.hash(&mut h);
        for row in cam_to_srgb {
            for v in row {
                v.to_bits().hash(&mut h);
            }
        }
        edits.lens.hash_key(&mut h);
        let wb = h.finish();

        let mut h = std::collections::hash_map::DefaultHasher::new();
        wb.hash(&mut h);
        hash_strokes(&edits.retouch, &mut h);
        edits.detail.hash_nr(&mut h);
        Self { wb, nr: h.finish() }
    }

    pub fn capture(&self, sigma: f32) -> u64 {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        self.nr.hash(&mut h);
        sigma.to_bits().hash(&mut h);
        h.finish()
    }

    pub fn spatial(&self, sigma: Option<f32>, spatial_dims: (u32, u32)) -> u64 {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        match sigma {
            Some(sigma) => self.capture(sigma).hash(&mut h),
            None => self.nr.hash(&mut h),
        }
        spatial_dims.0.hash(&mut h);
        spatial_dims.1.hash(&mut h);
        h.finish()
    }
}
