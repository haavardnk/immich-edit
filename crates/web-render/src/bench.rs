use half::f16;
use raw_pipeline::frame::FrameMeta;
use raw_pipeline::source::{LinearKind, SourceHeader, SourceImage};

pub fn synthetic_source(edge: u32) -> SourceImage {
    let width = edge.max(2);
    let height = (width * 2 / 3).max(1);
    let mut rgb_f16 = Vec::with_capacity(width as usize * height as usize * 3);
    for y in 0..height {
        for x in 0..width {
            let u = x as f32 / width as f32;
            let v = y as f32 / height as f32;
            let grain = ((x ^ y) & 63) as f32 / 63.0;
            for value in [u, v, 0.25 + 0.5 * grain] {
                rgb_f16.push(f16::from_f32(value).to_bits());
            }
        }
    }
    SourceImage {
        header: SourceHeader {
            meta: FrameMeta {
                width: width as usize,
                height: height as usize,
                wb_coeffs: [1.0; 4],
                xyz_to_cam: [
                    [1.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                    [0.0, 0.0, 1.0],
                    [0.0, 0.0, 0.0],
                ],
                color_matrices: Vec::new(),
                orientation: (false, false, false),
                is_raw: false,
                capture_sigma: None,
                model: String::new(),
            },
            kind: LinearKind::PostWb,
            dims: (width, height),
            atmosphere: None,
            window: None,
        },
        rgb_f16,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic_source_is_three_by_two_at_the_edge() {
        let image = synthetic_source(1600);
        if image.header.dims != (1600, 1066) {
            panic!("dims: {:?}", image.header.dims);
        }
        if image.rgb_f16.len() != 1600 * 1066 * 3 {
            panic!("samples: {}", image.rgb_f16.len());
        }
    }
}
