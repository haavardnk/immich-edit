use raw_pipeline::{CubeLut, LutMap};
use std::sync::Arc;

pub const TINT_LUT_ID: &str = "test";
pub const SHAPED_LUT_ID: &str = "shaped";

const SHAPER_SIZE: usize = 5000;

pub fn tint_luts() -> LutMap {
    let size = 16;
    let last = (size - 1) as f32;
    let mut cube = String::new();
    for (b, g, r) in
        (0..size).flat_map(|b| (0..size).flat_map(move |g| (0..size).map(move |r| (b, g, r))))
    {
        let rf = (r as f32 / last * 1.1).clamp(0.0, 1.0);
        let gf = g as f32 / last;
        let bf = (b as f32 / last * 0.85).clamp(0.0, 1.0);
        cube.push_str(&format!("{rf} {gf} {bf}\n"));
    }
    let shaper: String = (0..SHAPER_SIZE)
        .map(|i| {
            let v = i as f32 / (SHAPER_SIZE - 1) as f32;
            format!("{} {} {}\n", v.powf(0.8), v.powf(0.9), v.powf(1.2))
        })
        .collect();
    let tint = format!("LUT_3D_SIZE {size}\n{cube}");
    let shaped = format!(
        "LUT_1D_SIZE {SHAPER_SIZE}\nLUT_1D_INPUT_RANGE -0.1 1\nLUT_3D_SIZE {size}\n{shaper}{cube}"
    );
    LutMap::from([
        (TINT_LUT_ID.to_string(), parse(&tint)),
        (SHAPED_LUT_ID.to_string(), parse(&shaped)),
    ])
}

fn parse(src: &str) -> Arc<CubeLut> {
    Arc::new(CubeLut::parse(src.as_bytes()).unwrap())
}
