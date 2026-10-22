use rayon::prelude::*;

use crate::{PipelineError, PipelineResult};

#[derive(Debug, Clone, PartialEq)]
pub struct DenoisePlane {
    pub key: u64,
    pub width: usize,
    pub height: usize,
    pub rgb_f16: Vec<u16>,
}

impl DenoisePlane {
    pub fn new(key: u64, width: usize, height: usize, rgb_f16: Vec<u16>) -> PipelineResult<Self> {
        if width == 0 || height == 0 || rgb_f16.len() != width * height * 3 {
            return Err(PipelineError::Render(format!(
                "denoise plane holds {} samples, expected {width}x{height}x3",
                rgb_f16.len()
            )));
        }
        Ok(Self {
            key,
            width,
            height,
            rgb_f16,
        })
    }

    pub fn from_rgb(key: u64, width: usize, height: usize, rgb: &[f32]) -> PipelineResult<Self> {
        let rgb_f16 = rgb
            .par_iter()
            .map(|v| half::f16::from_f32(*v).to_bits())
            .collect();
        Self::new(key, width, height, rgb_f16)
    }

    pub fn binned_dims(&self, block: usize) -> (usize, usize) {
        (self.width / block, self.height / block)
    }

    pub fn binned_pixel(&self, x: usize, y: usize, block: usize) -> [f32; 3] {
        let span = x * block * 3..(x + 1) * block * 3;
        let sum = self
            .rgb_f16
            .chunks_exact(self.width * 3)
            .skip(y * block)
            .take(block)
            .flat_map(|row| row[span.clone()].chunks_exact(3))
            .fold([0.0f32; 3], |acc, px| {
                std::array::from_fn(|c| acc[c] + half::f16::from_bits(px[c]).to_f32())
            });
        let cells = (block * block) as f32;
        sum.map(|s| s / cells)
    }

    pub fn binned_rgb_f16(&self, block: usize) -> Vec<u16> {
        let (w, h) = self.binned_dims(block);
        (0..w * h)
            .into_par_iter()
            .flat_map_iter(|i| self.binned_pixel(i % w, i / w, block))
            .map(|v| half::f16::from_f32(v).to_bits())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plane_rejects_a_sample_count_that_does_not_match_its_dims() {
        assert!(DenoisePlane::from_rgb(1, 2, 2, &[0.0; 11]).is_err());
        assert!(DenoisePlane::new(1, 0, 2, Vec::new()).is_err());
        assert!(DenoisePlane::from_rgb(1, 2, 2, &[0.0; 12]).is_ok());
    }

    #[test]
    fn binning_averages_each_block_like_the_superpixel_grid() {
        let values: Vec<f32> = (0..4 * 2)
            .flat_map(|i| [i as f32 * 0.125, 0.5, 1.0 - i as f32 * 0.125])
            .collect();
        let plane = DenoisePlane::from_rgb(1, 4, 2, &values).unwrap();
        assert_eq!(plane.binned_dims(2), (2, 1));
        let left = plane.binned_pixel(0, 0, 2);
        let expected_red = (0.0 + 0.125 + 0.5 + 0.625) / 4.0;
        assert!((left[0] - expected_red).abs() < 1e-3, "{left:?}");
        assert!((left[1] - 0.5).abs() < 1e-3);
        assert_eq!(plane.binned_rgb_f16(2).len(), 2 * 3);
        assert_eq!(plane.binned_pixel(3, 1, 1)[0], 0.875);
    }
}
