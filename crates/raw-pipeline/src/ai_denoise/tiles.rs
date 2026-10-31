use rayon::prelude::*;

use crate::{PipelineError, PipelineResult};

pub struct Tiling {
    pub width: usize,
    pub height: usize,
    pub edge: usize,
    pub margin: usize,
    pub overlap: usize,
}

fn origins(len: usize, core: usize, overlap: usize) -> Vec<usize> {
    if len <= core {
        return vec![0];
    }
    let step = core.saturating_sub(overlap).max(1);
    let last = len - core;
    let mut out: Vec<usize> = (0..last).step_by(step).collect();
    out.push(last);
    out
}

fn mirror(at: isize, len: usize) -> usize {
    if len == 1 {
        return 0;
    }
    let period = 2 * (len as isize - 1);
    let folded = at.rem_euclid(period);
    if folded < len as isize {
        folded as usize
    } else {
        (period - folded) as usize
    }
}

fn ramp(t: usize, core: usize, overlap: usize, open_start: bool, open_end: bool) -> f32 {
    if overlap == 0 {
        return 1.0;
    }
    let width = overlap as f32;
    let rise = if open_start {
        1.0
    } else {
        (t as f32 + 0.5) / width
    };
    let fall = if open_end {
        1.0
    } else {
        (core as f32 - t as f32 - 0.5) / width
    };
    rise.min(fall).min(1.0)
}

fn axis_weights(origin: usize, len: usize, core: usize, overlap: usize) -> Vec<f32> {
    let open_start = origin == 0;
    let open_end = origin + core >= len;
    (0..core.min(len - origin))
        .map(|t| ramp(t, core, overlap, open_start, open_end))
        .collect()
}

fn extract(
    input: &impl Fn(usize) -> [f32; 3],
    (width, height): (usize, usize),
    (ox, oy): (usize, usize),
    edge: usize,
    margin: usize,
) -> Vec<f32> {
    let plane = edge * edge;
    let at = |origin: usize, t: usize| origin as isize + t as isize - margin as isize;
    let rows: Vec<usize> = (0..edge).map(|ty| mirror(at(oy, ty), height)).collect();
    let cols: Vec<usize> = (0..edge).map(|tx| mirror(at(ox, tx), width)).collect();
    let mut chw = vec![0.0f32; 3 * plane];
    for (ty, &sy) in rows.iter().enumerate() {
        for (tx, &sx) in cols.iter().enumerate() {
            let px = input(sy * width + sx);
            for (c, v) in px.into_iter().enumerate() {
                chw[c * plane + ty * edge + tx] = v;
            }
        }
    }
    chw
}

pub fn run_tiled<I, F, P>(
    tiling: Tiling,
    input: I,
    mut run: F,
    mut progress: P,
) -> PipelineResult<Vec<f32>>
where
    I: Fn(usize) -> [f32; 3],
    F: FnMut(&[f32]) -> PipelineResult<Vec<f32>>,
    P: FnMut(usize, usize),
{
    let Tiling {
        width,
        height,
        edge,
        margin,
        overlap,
    } = tiling;
    let core = edge.saturating_sub(2 * margin);
    if width == 0 || height == 0 || core == 0 || overlap * 2 > core {
        return Err(PipelineError::Render(format!(
            "tiling {width}x{height} at edge {edge} margin {margin} overlap {overlap}"
        )));
    }
    let xs = origins(width, core, overlap);
    let ys = origins(height, core, overlap);
    let total = xs.len() * ys.len();
    let plane = edge * edge;
    let mut sum = vec![0.0f32; width * height * 3];
    let mut weight = vec![0.0f32; width * height];
    let mut done = 0;
    progress(done, total);
    let tiles = ys.iter().flat_map(|&oy| xs.iter().map(move |&ox| (ox, oy)));
    for (ox, oy) in tiles {
        let wy = axis_weights(oy, height, core, overlap);
        let wx = axis_weights(ox, width, core, overlap);
        let out = run(&extract(&input, (width, height), (ox, oy), edge, margin))?;
        if out.len() != 3 * plane {
            return Err(PipelineError::Render(format!(
                "tile returned {} values, expected {}",
                out.len(),
                3 * plane
            )));
        }
        for (row, &row_weight) in wy.iter().enumerate() {
            let ty = row + margin;
            for (col, &col_weight) in wx.iter().enumerate() {
                let tx = col + margin;
                let w = row_weight * col_weight;
                let i = (oy + row) * width + ox + col;
                weight[i] += w;
                for c in 0..3 {
                    sum[i * 3 + c] += w * out[c * plane + ty * edge + tx];
                }
            }
        }
        done += 1;
        progress(done, total);
    }
    sum.par_chunks_exact_mut(3)
        .zip(weight.par_iter())
        .for_each(|(px, w)| px.iter_mut().for_each(|v| *v /= w.max(f32::MIN_POSITIVE)));
    Ok(sum)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ramp_image(width: usize, height: usize) -> Vec<f32> {
        (0..width * height)
            .flat_map(|i| {
                let x = (i % width) as f32;
                let y = (i / width) as f32;
                [x * 0.01, y * 0.02, (x + y) * 0.005]
            })
            .collect()
    }

    fn tiling(width: usize, height: usize, edge: usize, margin: usize, overlap: usize) -> Tiling {
        Tiling {
            width,
            height,
            edge,
            margin,
            overlap,
        }
    }

    fn pixels(rgb: &[f32]) -> impl Fn(usize) -> [f32; 3] + '_ {
        |i| [rgb[i * 3], rgb[i * 3 + 1], rgb[i * 3 + 2]]
    }

    #[test]
    fn identity_tiles_reassemble_the_input() {
        for (width, height) in [(10, 7), (37, 23), (64, 64), (100, 41), (1, 1)] {
            let rgb = ramp_image(width, height);
            let grid = tiling(width, height, 16, 2, 4);
            let out = run_tiled(grid, pixels(&rgb), |t| Ok(t.to_vec()), |_, _| {}).unwrap();
            let worst = rgb
                .iter()
                .zip(&out)
                .map(|(a, b)| (a - b).abs())
                .fold(0.0f32, f32::max);
            assert!(worst < 1e-5, "{width}x{height}: {worst}");
        }
    }

    #[test]
    fn a_constant_tile_result_is_reproduced_everywhere() {
        let rgb = ramp_image(50, 30);
        let constant = |t: &[f32]| Ok(vec![0.25; t.len()]);
        let out = run_tiled(tiling(50, 30, 16, 2, 6), pixels(&rgb), constant, |_, _| {}).unwrap();
        assert!(out.iter().all(|v| (v - 0.25).abs() < 1e-6));
    }

    #[test]
    fn no_pixel_takes_its_value_from_a_tile_margin() {
        let (edge, margin) = (16, 3);
        let marked = |t: &[f32]| {
            let plane = edge * edge;
            Ok((0..t.len())
                .map(|i| {
                    let (y, x) = ((i % plane) / edge, i % edge);
                    let inside = (margin..edge - margin).contains(&x)
                        && (margin..edge - margin).contains(&y);
                    if inside { 0.0 } else { 1.0 }
                })
                .collect())
        };
        for (width, height) in [(9, 5), (40, 23), (61, 61)] {
            let rgb = ramp_image(width, height);
            let grid = tiling(width, height, edge, margin, 4);
            let out = run_tiled(grid, pixels(&rgb), marked, |_, _| {}).unwrap();
            assert!(out.iter().all(|v| *v == 0.0), "{width}x{height}");
        }
    }

    #[test]
    fn border_tiles_see_mirrored_context_past_the_frame() {
        let width = 20;
        let rgb = ramp_image(width, 1);
        let mut first: Option<Vec<f32>> = None;
        run_tiled(
            tiling(width, 1, 16, 3, 4),
            pixels(&rgb),
            |t| {
                first.get_or_insert_with(|| t.to_vec());
                Ok(t.to_vec())
            },
            |_, _| {},
        )
        .unwrap();
        let reds = first.unwrap();
        assert_eq!(&reds[..4], &[rgb[9], rgb[6], rgb[3], rgb[0]]);
    }

    #[test]
    fn seams_are_feathered_across_the_overlap() {
        let width = 28;
        let mut calls = 0usize;
        let out = run_tiled(
            tiling(width, 1, 16, 2, 4),
            |_| [0.0; 3],
            |t| {
                calls += 1;
                Ok(vec![calls as f32; t.len()])
            },
            |_, _| {},
        )
        .unwrap();
        let reds: Vec<f32> = out.as_chunks::<3>().0.iter().map(|p| p[0]).collect();
        assert_eq!(reds[0], 1.0);
        assert_eq!(reds[width - 1], calls as f32);
        let blended = reds.iter().filter(|v| v.fract() != 0.0).count();
        assert!(blended >= 3, "{reds:?}");
        assert!(reds.windows(2).all(|w| w[1] >= w[0]), "{reds:?}");
    }

    #[test]
    fn progress_counts_every_tile_and_errors_stop_the_run() {
        let rgb = ramp_image(40, 20);
        let mut seen = Vec::new();
        run_tiled(
            tiling(40, 20, 16, 2, 4),
            pixels(&rgb),
            |t| Ok(t.to_vec()),
            |done, total| seen.push((done, total)),
        )
        .unwrap();
        let total = seen.last().unwrap().1;
        assert_eq!(seen.len(), total + 1);
        assert_eq!(seen.last().unwrap().0, total);

        let failed = run_tiled(
            tiling(40, 20, 16, 2, 4),
            pixels(&rgb),
            |_| Err(PipelineError::Cancelled),
            |_, _| {},
        );
        assert!(matches!(failed, Err(PipelineError::Cancelled)));
    }

    #[test]
    fn a_margin_that_leaves_no_core_is_rejected() {
        let rgb = ramp_image(8, 8);
        let result = run_tiled(
            tiling(8, 8, 16, 8, 0),
            pixels(&rgb),
            |t| Ok(t.to_vec()),
            |_, _| {},
        );
        assert!(matches!(result, Err(PipelineError::Render(_))));
    }

    #[test]
    fn mirror_padding_reflects_without_repeating_the_border() {
        assert_eq!(mirror(-1, 5), 1);
        assert_eq!(mirror(5, 5), 3);
        assert_eq!(mirror(9, 5), 1);
        assert_eq!(mirror(3, 1), 0);
    }
}
