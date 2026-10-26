use crate::refine::{RangeWindow, guided_filter, guided_radius, range_weight};

const LEVELS: usize = 256;
const REFINE_EPS: f32 = 1e-3;

fn level(value: f32) -> usize {
    (value.clamp(0.0, 1.0) * 255.0 + 0.5) as usize
}

pub fn window_by_rank(nearness: &[f32], window: RangeWindow) -> Vec<f32> {
    let mut counts = [0usize; LEVELS];
    nearness.iter().for_each(|v| counts[level(*v)] += 1);
    let total = nearness.len().max(1) as f32;
    let lo = window.min.min(window.max);
    let hi = window.min.max(window.max);
    let mut weights = [0.0f32; LEVELS];
    let mut covered = 0usize;
    for (slot, count) in weights.iter_mut().zip(counts).rev() {
        let start = covered as f32 / total;
        covered += count;
        let end = covered as f32 / total;
        let rank = if start <= lo && end >= hi { lo } else { end };
        *slot = range_weight(rank, window);
    }
    nearness.iter().map(|v| weights[level(*v)]).collect()
}

pub fn refine_depth(disparity: &[f32], guide: &[f32], w: usize, h: usize) -> Vec<f32> {
    guided_filter(guide, disparity, w, h, guided_radius(w, h), REFINE_EPS)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hard(min: f32, max: f32) -> RangeWindow {
        RangeWindow {
            min,
            max,
            softness: 0.0,
        }
    }

    #[test]
    fn window_selects_levels_by_their_nearest_share() {
        let cases = [
            (
                "full window keeps a flat map",
                vec![0.5; 4],
                hard(0.0, 1.0),
                vec![1.0; 4],
            ),
            (
                "far limit drops the farthest level first",
                vec![1.0, 2.0 / 3.0, 1.0 / 3.0, 0.0],
                hard(0.0, 0.8),
                vec![1.0, 1.0, 1.0, 0.0],
            ),
            (
                "near limit drops the nearest level first",
                vec![1.0, 2.0 / 3.0, 1.0 / 3.0, 0.0],
                hard(0.3, 1.0),
                vec![0.0, 1.0, 1.0, 1.0],
            ),
            (
                "meeting handles keep the level they fall in",
                vec![1.0, 0.0, 0.0, 0.0],
                hard(0.5, 0.5),
                vec![0.0, 1.0, 1.0, 1.0],
            ),
            (
                "a window inside one level keeps that level",
                vec![1.0, 0.0, 0.0, 0.0],
                hard(0.4, 0.6),
                vec![0.0, 1.0, 1.0, 1.0],
            ),
        ];
        for (name, nearness, window, want) in cases {
            assert_eq!(window_by_rank(&nearness, window), want, "{name}");
        }
    }

    #[test]
    fn window_matches_the_stored_eight_bit_map() {
        let nearness: Vec<f32> = (0..64).map(|i| (i as f32 * 0.37).fract()).collect();
        let stored: Vec<f32> = nearness.iter().map(|v| level(*v) as f32 / 255.0).collect();
        let window = hard(0.2, 0.7);
        assert_eq!(
            window_by_rank(&nearness, window),
            window_by_rank(&stored, window)
        );
    }

    #[test]
    fn refined_depth_snaps_to_the_image_edge() {
        let (w, h) = (96usize, 32usize);
        let edge = 48usize;
        let guide: Vec<f32> = (0..w * h)
            .map(|i| if i % w < edge { 0.8 } else { 0.2 })
            .collect();
        let coarse: Vec<f32> = (0..w * h)
            .map(|i| (((i % w) as f32 - edge as f32 + 12.0) / 24.0).clamp(0.0, 1.0))
            .map(|t| 1.0 - t)
            .collect();
        let refined = refine_depth(&coarse, &guide, w, h);
        let row = 16 * w;
        let step = |plane: &[f32]| plane[row + edge - 3] - plane[row + edge + 2];
        if step(&refined) < 2.0 * step(&coarse) {
            panic!(
                "edge step {} did not sharpen from {}",
                step(&refined),
                step(&coarse)
            );
        }
        let flat = refined[row + 4] - refined[row + w - 5];
        if flat < 0.9 {
            panic!("far ends lost their depth difference: {flat}");
        }
    }
}
