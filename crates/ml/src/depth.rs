use crate::refine::{RangeWindow, range_weight};

const LEVELS: usize = 256;

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
}
