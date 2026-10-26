use crate::edits::LensBlurEdits;
use crate::mask_raster::MaskRaster;

pub const COC_LEVELS: usize = 256;

#[derive(Clone, Debug, PartialEq)]
pub struct CocTable {
    pub values: [f32; COC_LEVELS],
}

struct Band {
    near: usize,
    far: usize,
    nearest: usize,
    farthest: usize,
}

fn focus_band(counts: &[usize; COC_LEVELS], lo: f32, hi: f32) -> Option<Band> {
    let total = counts.iter().sum::<usize>().max(1) as f32;
    let mut covered = 0usize;
    let mut near = None;
    let mut far = None;
    let mut nearest = None;
    let mut farthest = 0usize;
    for (level, count) in counts.iter().enumerate().rev() {
        if *count == 0 {
            continue;
        }
        nearest.get_or_insert(level);
        farthest = level;
        let start = covered as f32 / total;
        covered += count;
        let end = covered as f32 / total;
        let overlaps = end > lo && start < hi;
        let holds = start <= lo && end >= hi;
        if overlaps || holds {
            near.get_or_insert(level);
            far = Some(level);
        }
    }
    Some(Band {
        near: near?,
        far: far?,
        nearest: nearest?,
        farthest,
    })
}

impl CocTable {
    pub fn new(raster: &MaskRaster, edits: &LensBlurEdits) -> Self {
        let mut counts = [0usize; COC_LEVELS];
        raster.bytes.iter().for_each(|b| counts[*b as usize] += 1);
        Self::from_counts(&counts, edits.focus_min as f32, edits.focus_max as f32)
    }

    fn from_counts(counts: &[usize; COC_LEVELS], lo: f32, hi: f32) -> Self {
        let mut values = [0.0f32; COC_LEVELS];
        let Some(band) = focus_band(counts, lo.min(hi), lo.max(hi)) else {
            return Self { values };
        };
        let span = (band.nearest - band.near).max(band.far - band.farthest);
        if span == 0 {
            return Self { values };
        }
        let norm = span as f32;
        for (level, slot) in values.iter_mut().enumerate() {
            let signed = if level > band.near {
                (level - band.near) as f32
            } else if level < band.far {
                -((band.far - level) as f32)
            } else {
                0.0
            };
            *slot = (signed / norm).clamp(-1.0, 1.0);
        }
        Self { values }
    }

    #[inline(always)]
    pub fn sample(&self, depth: f32) -> f32 {
        let f = depth.clamp(0.0, 1.0) * (COC_LEVELS - 1) as f32;
        let i = (f.floor() as usize).min(COC_LEVELS - 2);
        let t = f - i as f32;
        self.values[i] + (self.values[i + 1] - self.values[i]) * t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn counts(levels: &[(usize, usize)]) -> [usize; COC_LEVELS] {
        let mut out = [0usize; COC_LEVELS];
        for (level, count) in levels {
            out[*level] = *count;
        }
        out
    }

    #[test]
    fn the_focus_band_is_sharp_and_the_far_extreme_is_full_blur() {
        let hist = counts(&[(250, 30), (180, 20), (100, 25), (20, 25)]);
        let table = CocTable::from_counts(&hist, 0.0, 0.3);
        assert_eq!(table.values[250], 0.0);
        assert_eq!(table.values[20], -1.0);
        assert!(table.values[100] < 0.0 && table.values[100] > -1.0);
        assert_eq!(table.values[255], 5.0 / 230.0);
    }

    #[test]
    fn a_far_focus_blurs_the_nearest_level_fully() {
        let hist = counts(&[(250, 30), (180, 20), (100, 25), (20, 25)]);
        let table = CocTable::from_counts(&hist, 0.8, 1.0);
        assert_eq!(table.values[20], 0.0);
        assert_eq!(table.values[250], 1.0);
        assert!(table.values[180] > 0.0 && table.values[180] < 1.0);
    }

    #[test]
    fn the_table_rises_with_nearness() {
        let hist = counts(&[(240, 10), (160, 40), (90, 30), (10, 20)]);
        let table = CocTable::from_counts(&hist, 0.2, 0.5);
        for pair in table.values.windows(2) {
            assert!(pair[1] >= pair[0], "{pair:?}");
        }
        assert_eq!(table.values[160], 0.0);
    }

    #[test]
    fn a_window_covering_everything_keeps_the_frame_sharp() {
        let hist = counts(&[(200, 10), (50, 10)]);
        let table = CocTable::from_counts(&hist, 0.0, 1.0);
        assert!(table.values.iter().all(|v| *v == 0.0));
    }

    #[test]
    fn sampling_interpolates_between_levels() {
        let hist = counts(&[(255, 1), (0, 1)]);
        let table = CocTable::from_counts(&hist, 0.0, 0.4);
        let mid = table.sample(127.5 / 255.0);
        let want = (table.values[127] + table.values[128]) / 2.0;
        assert!((mid - want).abs() < 1e-6, "{mid} {want}");
    }
}
