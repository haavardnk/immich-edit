use raw_pipeline::{Lut3d, LutMap};
use std::sync::Arc;

pub const TINT_LUT_ID: &str = "test";

pub fn tint_luts() -> LutMap {
    let size = 16;
    let last = (size - 1) as f32;
    let mut cube = format!("LUT_3D_SIZE {size}\n");
    for (b, g, r) in
        (0..size).flat_map(|b| (0..size).flat_map(move |g| (0..size).map(move |r| (b, g, r))))
    {
        let rf = (r as f32 / last * 1.1).clamp(0.0, 1.0);
        let gf = g as f32 / last;
        let bf = (b as f32 / last * 0.85).clamp(0.0, 1.0);
        cube.push_str(&format!("{rf} {gf} {bf}\n"));
    }
    let lut = Lut3d::parse_cube(cube.as_bytes()).unwrap();
    LutMap::from([(TINT_LUT_ID.to_string(), Arc::new(lut))])
}
