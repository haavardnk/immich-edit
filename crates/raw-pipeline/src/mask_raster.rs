use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug)]
pub struct MaskRaster {
    pub width: u32,
    pub height: u32,
    pub bytes: Vec<u8>,
}

impl MaskRaster {
    pub fn new(width: u32, height: u32, bytes: Vec<u8>) -> Option<Self> {
        if width == 0 || height == 0 {
            return None;
        }
        if bytes.len() != (width as usize) * (height as usize) {
            return None;
        }
        Some(Self {
            width,
            height,
            bytes,
        })
    }

    #[inline(always)]
    pub fn sample_bilinear(&self, u: f32, v: f32) -> f32 {
        let w = self.width as i32;
        let h = self.height as i32;
        if w <= 0 || h <= 0 {
            return 0.0;
        }
        let fx = (u.clamp(0.0, 1.0) * w as f32 - 0.5).max(0.0);
        let fy = (v.clamp(0.0, 1.0) * h as f32 - 0.5).max(0.0);
        let x0 = (fx.floor() as i32).clamp(0, w - 1);
        let y0 = (fy.floor() as i32).clamp(0, h - 1);
        let x1 = (x0 + 1).min(w - 1);
        let y1 = (y0 + 1).min(h - 1);
        let tx = fx - x0 as f32;
        let ty = fy - y0 as f32;
        let row0 = (y0 as usize) * (w as usize);
        let row1 = (y1 as usize) * (w as usize);
        let a = self.bytes[row0 + x0 as usize] as f32;
        let b = self.bytes[row0 + x1 as usize] as f32;
        let c = self.bytes[row1 + x0 as usize] as f32;
        let d = self.bytes[row1 + x1 as usize] as f32;
        let top = a + (b - a) * tx;
        let bot = c + (d - c) * tx;
        (top + (bot - top) * ty) / 255.0
    }
}

pub type RasterMap = HashMap<String, Arc<MaskRaster>>;

pub fn empty_rasters() -> RasterMap {
    HashMap::new()
}
