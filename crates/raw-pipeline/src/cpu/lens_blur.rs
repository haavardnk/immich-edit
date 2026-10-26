use rayon::prelude::*;

use crate::math::smoothstep;
use crate::ops::LinearImage;
use crate::ops::lens_blur::LensBlur;
use crate::ops::lens_blur::kernel::{
    aperture_distance, gather_weight, highlight_weight, sender_reach, upsample_weight,
};
use crate::ops::lens_distortion::{LensWarpParams, mask_uv_to_scene_uv};

struct Grid {
    cells: Vec<[f32; 4]>,
    width: usize,
    height: usize,
    ds: usize,
}

pub struct LensBlurFrame<'a> {
    blur: &'a LensBlur,
    coc: Vec<f32>,
    distances: Vec<f32>,
}

impl<'a> LensBlurFrame<'a> {
    pub fn new(blur: &'a LensBlur, warp: &LensWarpParams, (w, h): (usize, usize)) -> Option<Self> {
        if w == 0 || h == 0 {
            return None;
        }
        Some(Self {
            blur,
            coc: signed_coc(w, h, blur, warp),
            distances: aperture_distances(blur),
        })
    }

    pub fn apply(&self, image: &mut LinearImage) {
        let grid = downsample(image, &self.coc, self.blur.scale.ds as usize);
        let blurred = gather(&grid, self.blur, &self.distances);
        composite(image, &self.coc, &grid, &blurred);
    }
}

fn signed_coc(w: usize, h: usize, blur: &LensBlur, warp: &LensWarpParams) -> Vec<f32> {
    let mut out = vec![0.0f32; w * h];
    out.par_chunks_exact_mut(w)
        .enumerate()
        .for_each(|(y, row)| {
            let v = (y as f32 + 0.5) / h as f32;
            for (x, slot) in row.iter_mut().enumerate() {
                let u = (x as f32 + 0.5) / w as f32;
                let [su, sv] = mask_uv_to_scene_uv(warp, [u, v]);
                let depth = blur.depth.sample_bilinear(su, sv);
                *slot = blur.scale.radius * blur.table.sample(depth);
            }
        });
    out
}

fn aperture_distances(blur: &LensBlur) -> Vec<f32> {
    let reach = blur.scale.reach;
    (-reach..=reach)
        .flat_map(|dy| (-reach..=reach).map(move |dx| (dx, dy)))
        .map(|(dx, dy)| aperture_distance(dx as f32, dy as f32, blur.blades))
        .collect()
}

fn downsample(image: &LinearImage, coc: &[f32], ds: usize) -> Grid {
    let (w, h) = (image.width, image.height);
    let (gw, gh) = (w.div_ceil(ds), h.div_ceil(ds));
    let mut cells = vec![[0.0f32; 4]; gw * gh];
    cells
        .par_chunks_exact_mut(gw)
        .enumerate()
        .for_each(|(gy, row)| {
            for (gx, cell) in row.iter_mut().enumerate() {
                let mut acc = [0.0f32; 4];
                let mut count = 0.0f32;
                for y in gy * ds..((gy + 1) * ds).min(h) {
                    for x in gx * ds..((gx + 1) * ds).min(w) {
                        let i = y * w + x;
                        acc[0] += image.rgb[i * 3];
                        acc[1] += image.rgb[i * 3 + 1];
                        acc[2] += image.rgb[i * 3 + 2];
                        acc[3] += coc[i];
                        count += 1.0;
                    }
                }
                *cell = [
                    acc[0] / count,
                    acc[1] / count,
                    acc[2] / count,
                    acc[3] / count / ds as f32,
                ];
            }
        });
    Grid {
        cells,
        width: gw,
        height: gh,
        ds,
    }
}

fn gather(grid: &Grid, blur: &LensBlur, distances: &[f32]) -> Vec<[f32; 4]> {
    let (gw, gh) = (grid.width as i32, grid.height as i32);
    let reach = blur.scale.reach;
    let side = (2 * reach + 1) as usize;
    let mut out = vec![[0.0f32; 4]; grid.cells.len()];
    out.par_chunks_exact_mut(grid.width)
        .enumerate()
        .for_each(|(py, row)| {
            for (px, slot) in row.iter_mut().enumerate() {
                let receiver = grid.cells[py * grid.width + px][3];
                let mut acc = [0.0f32; 4];
                let mut total = 0.0f32;
                for dy in -reach..=reach {
                    let qy = py as i32 + dy;
                    if qy < 0 || qy >= gh {
                        continue;
                    }
                    for dx in -reach..=reach {
                        let qx = px as i32 + dx;
                        if qx < 0 || qx >= gw {
                            continue;
                        }
                        let q = grid.cells[(qy * gw + qx) as usize];
                        let distance =
                            distances[(dy + reach) as usize * side + (dx + reach) as usize];
                        let spread = sender_reach(receiver, q[3]);
                        let weight = gather_weight(spread, distance);
                        if weight <= 0.0 {
                            continue;
                        }
                        let weight = weight * highlight_weight([q[0], q[1], q[2]], blur.boost);
                        acc[0] += q[0] * weight;
                        acc[1] += q[1] * weight;
                        acc[2] += q[2] * weight;
                        acc[3] += spread * weight;
                        total += weight;
                    }
                }
                *slot = [
                    acc[0] / total,
                    acc[1] / total,
                    acc[2] / total,
                    acc[3] / total,
                ];
            }
        });
    out
}

fn composite(image: &mut LinearImage, coc: &[f32], grid: &Grid, blurred: &[[f32; 4]]) {
    let w = image.width;
    let ds = grid.ds as f32;
    let (gw, gh) = (grid.width as i32, grid.height as i32);
    image
        .rgb
        .par_chunks_exact_mut(w * 3)
        .enumerate()
        .for_each(|(y, row)| {
            let fy = (y as f32 + 0.5) / ds - 0.5;
            let y0 = fy.floor();
            let ty = fy - y0;
            let rows = [
                (y0 as i32).clamp(0, gh - 1),
                (y0 as i32 + 1).clamp(0, gh - 1),
            ];
            for (x, px) in row.chunks_exact_mut(3).enumerate() {
                let pixel = coc[y * w + x];
                let fx = (x as f32 + 0.5) / ds - 0.5;
                let x0 = fx.floor();
                let tx = fx - x0;
                let cols = [
                    (x0 as i32).clamp(0, gw - 1),
                    (x0 as i32 + 1).clamp(0, gw - 1),
                ];
                let mut acc = [0.0f32; 4];
                let mut total = 0.0f32;
                for (ry, row_index) in rows.iter().enumerate() {
                    let wy = if ry == 0 { 1.0 - ty } else { ty };
                    for (cx, col_index) in cols.iter().enumerate() {
                        let wx = if cx == 0 { 1.0 - tx } else { tx };
                        let cell = (row_index * gw + col_index) as usize;
                        let weight = wx * wy * upsample_weight(grid.cells[cell][3], pixel / ds);
                        let b = blurred[cell];
                        acc[0] += b[0] * weight;
                        acc[1] += b[1] * weight;
                        acc[2] += b[2] * weight;
                        acc[3] += b[3] * weight;
                        total += weight;
                    }
                }
                let mix = smoothstep(0.0, 1.0, (pixel.abs() / ds).max(acc[3] / total));
                if mix <= 0.0 {
                    continue;
                }
                for (c, value) in px.iter_mut().enumerate() {
                    *value += (acc[c] / total - *value) * mix;
                }
            }
        });
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::edits::{Edits, LensBlurEdits};
    use crate::mask_raster::{MaskRaster, RasterMap};

    const SIZE: usize = 512;

    fn inside(x: usize, y: usize) -> bool {
        (200..312).contains(&x) && (200..312).contains(&y)
    }

    #[test]
    fn a_blurred_foreground_spreads_over_the_sharp_background() {
        let rgb = (0..SIZE * SIZE)
            .flat_map(|i| {
                let v = if inside(i % SIZE, i / SIZE) { 1.0 } else { 0.2 };
                [v, v, v]
            })
            .collect();
        let mut image = LinearImage::new(rgb, SIZE, SIZE);
        let depth: Vec<u8> = (0..SIZE * SIZE)
            .map(|i| if inside(i % SIZE, i / SIZE) { 255 } else { 100 })
            .collect();
        let rasters = RasterMap::from([(
            "depth".to_string(),
            Arc::new(MaskRaster::new(SIZE as u32, SIZE as u32, depth).unwrap()),
        )]);
        let edits = Edits {
            lens_blur: LensBlurEdits {
                amount: 100.0,
                focus_min: 0.5,
                focus_max: 1.0,
                depth_raster_id: "depth".into(),
                ..LensBlurEdits::default()
            },
            ..Edits::default()
        };
        let blur = LensBlur::resolve(&edits, &rasters, (SIZE as u32, SIZE as u32)).unwrap();
        let warp = LensWarpParams::from_edits(&Default::default(), SIZE as u32, SIZE as u32);
        LensBlurFrame::new(&blur, &warp, (SIZE, SIZE))
            .unwrap()
            .apply(&mut image);
        let at = |x: usize, y: usize| image.rgb[(y * SIZE + x) * 3];
        if at(196, 256) < 0.3 {
            panic!(
                "the foreground did not spread past its edge: {}",
                at(196, 256)
            );
        }
        if at(256, 256) < 0.9 {
            panic!("the foreground centre moved: {}", at(256, 256));
        }
        if (at(40, 40) - 0.2).abs() > 1e-6 {
            panic!("the far sharp background changed: {}", at(40, 40));
        }
    }
}
