use std::sync::Arc;

use raw_pipeline::edits::{BokehShape, Edits, LensBlurEdits};
use raw_pipeline::frame::RawFrame;
use raw_pipeline::mask_raster::{MaskRaster, RasterMap};

use crate::frames::rgb_frame;

pub const DEPTH_ID: &str = "depth";

pub fn depth_raster(w: u32, h: u32) -> MaskRaster {
    let bytes = (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .map(|(x, y)| {
            let u = (x as f32 + 0.5) / w as f32 - 0.45;
            let v = (y as f32 + 0.5) / h as f32 - 0.5;
            if u * u + v * v < 0.04 {
                240
            } else {
                (170.0 - 150.0 * (y as f32 / (h - 1) as f32)) as u8
            }
        })
        .collect();
    MaskRaster::new(w, h, bytes).expect("depth raster dims")
}

pub fn depth_rasters() -> RasterMap {
    RasterMap::from([(DEPTH_ID.to_string(), Arc::new(depth_raster(96, 64)))])
}

pub fn lens_blur_edits(amount: f64, shape: BokehShape, boost: f64) -> Edits {
    Edits {
        lens_blur: LensBlurEdits {
            amount,
            focus_min: 0.0,
            focus_max: 0.15,
            shape,
            boost,
            depth_raster_id: DEPTH_ID.into(),
        },
        ..Edits::default()
    }
}

pub fn bokeh_frame(w: usize, h: usize) -> RawFrame {
    let mut data = vec![0.0f32; w * h * 3];
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) * 3;
            let checker = if (x / 3 + y / 3) % 2 == 0 {
                0.08
            } else {
                -0.08
            };
            let base = 0.2 + 0.3 * (x as f32 / (w - 1) as f32);
            let spot = (x % 41 < 3 && y % 37 < 3) as u8 as f32 * 3.0;
            data[i] = base + checker + spot;
            data[i + 1] = base * 0.9 - checker + spot;
            data[i + 2] = base * 0.7 + checker + spot * 0.8;
        }
    }
    rgb_frame(w, h, data)
}
