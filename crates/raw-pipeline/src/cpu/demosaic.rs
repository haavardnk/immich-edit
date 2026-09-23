use crate::tone::shared::RAW_LINEAR_CEILING;
use multiversion::multiversion;
use rayon::prelude::*;

fn cfa_channel(cfa: &[u8; 4], x: usize, y: usize) -> usize {
    let c = cfa[(y & 1) * 2 + (x & 1)];
    match c {
        b'R' => 0,
        b'G' => 1,
        b'B' => 2,
        _ => 1,
    }
}

pub fn bilinear(data: &[f32], w: usize, h: usize, cfa_pattern: &str) -> Vec<f32> {
    let cfa = parse_cfa(cfa_pattern);
    let mut out = vec![0.0f32; w * h * 3];
    out.par_chunks_mut(w * 3).enumerate().for_each(|(y, row)| {
        for x in 0..w {
            let rgb = bilinear_pixel(data, w, h, &cfa, x, y);
            let off = x * 3;
            row[off] = rgb[0];
            row[off + 1] = rgb[1];
            row[off + 2] = rgb[2];
        }
    });
    out
}

fn parse_cfa(cfa_pattern: &str) -> [u8; 4] {
    let mut cfa = *b"RGGB";
    for (i, b) in cfa_pattern.bytes().take(4).enumerate() {
        cfa[i] = b;
    }
    cfa
}

#[inline]
fn bilinear_pixel(data: &[f32], w: usize, h: usize, cfa: &[u8; 4], x: usize, y: usize) -> [f32; 3] {
    let own_ch = cfa_channel(cfa, x, y);
    let mut rgb = [0.0f32; 3];
    rgb[own_ch] = data[y * w + x];
    for (ch, slot) in rgb.iter_mut().enumerate() {
        if ch == own_ch {
            continue;
        }
        let mut sum = 0.0f32;
        let mut count = 0u32;
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                    continue;
                }
                if cfa_channel(cfa, nx as usize, ny as usize) == ch {
                    sum += data[ny as usize * w + nx as usize];
                    count += 1;
                }
            }
        }
        if count > 0 {
            *slot = sum / count as f32;
        }
    }
    rgb
}

pub fn malvar_he_cutler(data: &[f32], w: usize, h: usize, cfa_pattern: &str) -> Vec<f32> {
    if w < 5 || h < 5 {
        return bilinear(data, w, h, cfa_pattern);
    }
    let cfa = parse_cfa(cfa_pattern);
    let mut out = vec![0.0f32; w * h * 3];
    out.par_chunks_exact_mut(w * 3).enumerate().for_each_init(
        || MhcKernels::new(w - 4),
        |kernels, (y, row)| mhc_row(data, w, h, &cfa, y, row, kernels),
    );
    out
}

struct MhcKernels {
    h: Vec<f32>,
    v: Vec<f32>,
    g: Vec<f32>,
    opp: Vec<f32>,
}

impl MhcKernels {
    fn new(n: usize) -> Self {
        Self {
            h: vec![0.0; n],
            v: vec![0.0; n],
            g: vec![0.0; n],
            opp: vec![0.0; n],
        }
    }
}

#[multiversion(targets("x86_64+avx2", "x86_64+sse4.2"))]
fn mhc_row(
    data: &[f32],
    w: usize,
    h: usize,
    cfa: &[u8; 4],
    y: usize,
    row: &mut [f32],
    k: &mut MhcKernels,
) {
    let bilinear_at = |row: &mut [f32], x: usize| {
        let rgb = bilinear_pixel(data, w, h, cfa, x, y);
        row[x * 3..x * 3 + 3].copy_from_slice(&rgb);
    };
    if y < 2 || y >= h - 2 {
        (0..w).for_each(|x| bilinear_at(row, x));
        return;
    }
    let n = w - 4;
    let line = |dy: usize| &data[(y + dy - 2) * w..][..w];
    let up2 = &line(0)[2..n + 2];
    let down2 = &line(4)[2..n + 2];
    let up = line(1);
    let down = line(3);
    let mid = line(2);
    let ul = &up[1..n + 1];
    let uc = &up[2..n + 2];
    let ur = &up[3..n + 3];
    let dl = &down[1..n + 1];
    let dc = &down[2..n + 2];
    let dr = &down[3..n + 3];
    let m0 = &mid[..n];
    let m1 = &mid[1..n + 1];
    let m2 = &mid[2..n + 2];
    let m3 = &mid[3..n + 3];
    let m4 = &mid[4..n + 4];
    let kh = &mut k.h[..n];
    let kv = &mut k.v[..n];
    let kg = &mut k.g[..n];
    let kopp = &mut k.opp[..n];
    for i in 0..n {
        let c = m2[i];
        let n1 = m1[i] + m3[i];
        let n2 = uc[i] + dc[i];
        let d2 = m0[i] + m4[i];
        let d2v = up2[i] + down2[i];
        let diag = ul[i] + ur[i] + dl[i] + dr[i];
        kh[i] = ((n1 * 4.0 + c * 5.0 - d2 - diag + d2v * 0.5) / 8.0).clamp(0.0, RAW_LINEAR_CEILING);
        kv[i] = ((n2 * 4.0 + c * 5.0 - d2v - diag + d2 * 0.5) / 8.0).clamp(0.0, RAW_LINEAR_CEILING);
        let n4 = m1[i] + m3[i] + uc[i] + dc[i];
        let dplus = m0[i] + m4[i] + up2[i] + down2[i];
        kg[i] = ((n4 * 2.0 + c * 4.0 - dplus) / 8.0).clamp(0.0, RAW_LINEAR_CEILING);
        kopp[i] = ((diag * 2.0 + c * 6.0 - (m0[i] + m4[i] + up2[i] + down2[i]) * 1.5) / 8.0)
            .clamp(0.0, RAW_LINEAR_CEILING);
    }
    let class = |parity: usize| {
        (
            cfa_channel(cfa, parity, y),
            cfa_channel(cfa, parity + 1, y),
            cfa_channel(cfa, parity, y + 1),
        )
    };
    let classes = [class(0), class(1)];
    for (i, px) in row[6..(w - 2) * 3].chunks_exact_mut(3).enumerate() {
        let (own, row_ch, col_ch) = classes[i & 1];
        if own == 1 {
            px[row_ch] = kh[i];
            px[col_ch] = kv[i];
            px[1] = m2[i];
        } else {
            px[1] = kg[i];
            px[2 - own] = kopp[i];
            px[own] = m2[i];
        }
    }
    [0, 1, w - 2, w - 1]
        .into_iter()
        .for_each(|x| bilinear_at(row, x));
}

const XTRANS_DIM: usize = 6;
const XTRANS_LEN: usize = XTRANS_DIM * XTRANS_DIM;

const GREEN_NEIGHBORS: [(i32, i32, f32); 8] = [
    (-1, 0, 1.0),
    (1, 0, 1.0),
    (0, -1, 1.0),
    (0, 1, 1.0),
    (-1, -1, 0.5),
    (1, -1, 0.5),
    (-1, 1, 0.5),
    (1, 1, 0.5),
];

pub fn parse_xtrans(cfa_pattern: &str) -> Option<[u8; XTRANS_LEN]> {
    let bytes = cfa_pattern.as_bytes();
    if bytes.len() != XTRANS_LEN || bytes.iter().any(|b| !matches!(b, b'R' | b'G' | b'B')) {
        return None;
    }
    let mut pattern = [b'G'; XTRANS_LEN];
    pattern.copy_from_slice(bytes);
    Some(pattern)
}

fn xtrans_channel(pattern: &[u8; XTRANS_LEN], x: usize, y: usize) -> usize {
    match pattern[(y % XTRANS_DIM) * XTRANS_DIM + x % XTRANS_DIM] {
        b'R' => 0,
        b'B' => 2,
        _ => 1,
    }
}

fn xtrans_green(data: &[f32], w: usize, h: usize, pattern: &[u8; XTRANS_LEN]) -> Vec<f32> {
    let taps = XtransTaps::new(pattern, w);
    let mut green = vec![0.0f32; w * h];
    green
        .par_chunks_mut(w)
        .enumerate()
        .for_each(|(y, row)| xtrans_green_row(data, w, h, pattern, &taps, y, row));
    green
}

fn xtrans_green_row(
    data: &[f32],
    w: usize,
    h: usize,
    pattern: &[u8; XTRANS_LEN],
    taps: &XtransTaps,
    y: usize,
    row: &mut [f32],
) {
    let interior_y = y >= 1 && y + 1 < h;
    for (x, slot) in row.iter_mut().enumerate() {
        let i = y * w + x;
        if xtrans_channel(pattern, x, y) == 1 {
            *slot = data[i];
            continue;
        }
        let (sum, weight) = if interior_y && x >= 1 && x + 1 < w {
            taps.green[xtrans_phase(x, y)].iter().fold(
                (0.0f32, 0.0f32),
                |(sum, weight), &(offset, wgt)| {
                    (
                        sum + data[i.wrapping_add_signed(offset)] * wgt,
                        weight + wgt,
                    )
                },
            )
        } else {
            xtrans_green_edge(data, w, h, pattern, x, y)
        };
        *slot = if weight > 0.0 {
            (sum / weight).clamp(0.0, RAW_LINEAR_CEILING)
        } else {
            data[i]
        };
    }
}

fn xtrans_green_edge(
    data: &[f32],
    w: usize,
    h: usize,
    pattern: &[u8; XTRANS_LEN],
    x: usize,
    y: usize,
) -> (f32, f32) {
    let mut sum = 0.0f32;
    let mut weight = 0.0f32;
    for (dx, dy, wgt) in GREEN_NEIGHBORS {
        let nx = x as i32 + dx;
        let ny = y as i32 + dy;
        if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
            continue;
        }
        let nx = nx as usize;
        let ny = ny as usize;
        if xtrans_channel(pattern, nx, ny) != 1 {
            continue;
        }
        sum += data[ny * w + nx] * wgt;
        weight += wgt;
    }
    (sum, weight)
}

fn xtrans_phase(x: usize, y: usize) -> usize {
    (y % XTRANS_DIM) * XTRANS_DIM + x % XTRANS_DIM
}

struct XtransTaps {
    green: Vec<Vec<(isize, f32)>>,
    chroma: Vec<[Vec<(isize, f32)>; 2]>,
}

impl XtransTaps {
    fn new(pattern: &[u8; XTRANS_LEN], w: usize) -> Self {
        let offset = |dx: i32, dy: i32| dy as isize * w as isize + dx as isize;
        let channel_at = |phase: usize, dx: i32, dy: i32| {
            let x = (phase % XTRANS_DIM) as i32 + dx + XTRANS_DIM as i32;
            let y = (phase / XTRANS_DIM) as i32 + dy + XTRANS_DIM as i32;
            xtrans_channel(pattern, x as usize, y as usize)
        };
        let green = (0..XTRANS_LEN)
            .map(|phase| {
                GREEN_NEIGHBORS
                    .iter()
                    .filter(|&&(dx, dy, _)| channel_at(phase, dx, dy) == 1)
                    .map(|&(dx, dy, wgt)| (offset(dx, dy), wgt))
                    .collect()
            })
            .collect();
        let window = (-2i32..=2).flat_map(|dy| (-2i32..=2).map(move |dx| (dx, dy)));
        let chroma = (0..XTRANS_LEN)
            .map(|phase| {
                [0usize, 2].map(|ch| {
                    window
                        .clone()
                        .filter(|&(dx, dy)| channel_at(phase, dx, dy) == ch)
                        .map(|(dx, dy)| (offset(dx, dy), 1.0 / ((dx * dx + dy * dy) as f32)))
                        .collect()
                })
            })
            .collect();
        Self { green, chroma }
    }
}

fn xtrans_chroma(
    data: &[f32],
    green: &[f32],
    dim: (usize, usize),
    pattern: &[u8; XTRANS_LEN],
    pos: (usize, usize),
    ch: usize,
) -> f32 {
    let (w, h) = dim;
    let (x, y) = pos;
    let mut sum = 0.0f32;
    let mut weight = 0.0f32;
    for dy in -2i32..=2 {
        for dx in -2i32..=2 {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                continue;
            }
            let nx = nx as usize;
            let ny = ny as usize;
            if xtrans_channel(pattern, nx, ny) != ch {
                continue;
            }
            let wgt = 1.0 / ((dx * dx + dy * dy) as f32);
            let n = ny * w + nx;
            sum += (data[n] - green[n]) * wgt;
            weight += wgt;
        }
    }
    if weight > 0.0 { sum / weight } else { 0.0 }
}

pub fn xtrans(data: &[f32], w: usize, h: usize, pattern: &[u8; XTRANS_LEN]) -> Vec<f32> {
    let green = xtrans_green(data, w, h, pattern);
    let taps = XtransTaps::new(pattern, w);
    let mut out = vec![0.0f32; w * h * 3];
    out.par_chunks_mut(w * 3)
        .enumerate()
        .for_each(|(y, row)| xtrans_rgb_row(data, &green, w, h, pattern, &taps, y, row));
    out
}

#[allow(clippy::too_many_arguments)]
fn xtrans_rgb_row(
    data: &[f32],
    green: &[f32],
    w: usize,
    h: usize,
    pattern: &[u8; XTRANS_LEN],
    taps: &XtransTaps,
    y: usize,
    row: &mut [f32],
) {
    let interior_y = y >= 2 && y + 2 < h;
    for (x, px) in row.chunks_exact_mut(3).enumerate() {
        let i = y * w + x;
        let own = xtrans_channel(pattern, x, y);
        px[1] = green[i];
        if own != 1 {
            px[own] = data[i];
        }
        let interior = interior_y && x >= 2 && x + 2 < w;
        for (slot, ch) in [(0usize, 0usize), (1, 2)] {
            if ch == own {
                continue;
            }
            let chroma = if interior {
                let (sum, weight) = taps.chroma[xtrans_phase(x, y)][slot].iter().fold(
                    (0.0f32, 0.0f32),
                    |(sum, weight), &(offset, wgt)| {
                        let n = i.wrapping_add_signed(offset);
                        (sum + (data[n] - green[n]) * wgt, weight + wgt)
                    },
                );
                if weight > 0.0 { sum / weight } else { 0.0 }
            } else {
                xtrans_chroma(data, green, (w, h), pattern, (x, y), ch)
            };
            px[ch] = (green[i] + chroma).clamp(0.0, RAW_LINEAR_CEILING);
        }
    }
}

pub fn superpixel(data: &[f32], w: usize, h: usize, cfa_pattern: &str, block: usize) -> Vec<f32> {
    match parse_xtrans(cfa_pattern) {
        Some(pattern) => superpixel_with(data, w, h, block, |x, y| xtrans_channel(&pattern, x, y)),
        None => {
            let cfa = parse_cfa(cfa_pattern);
            superpixel_with(data, w, h, block, |x, y| cfa_channel(&cfa, x, y))
        }
    }
}

fn superpixel_with(
    data: &[f32],
    w: usize,
    h: usize,
    block: usize,
    channel: impl Fn(usize, usize) -> usize + Sync,
) -> Vec<f32> {
    let out_w = w / block;
    let out_h = h / block;
    let mut out = vec![0.0f32; out_w * out_h * 3];
    out.par_chunks_exact_mut(out_w * 3)
        .enumerate()
        .for_each(|(by, row)| {
            for (bx, px) in row.chunks_exact_mut(3).enumerate() {
                let (sum, count) =
                    (0..block * block).fold(([0.0f32; 3], [0u32; 3]), |(mut sum, mut count), i| {
                        let x = bx * block + i % block;
                        let y = by * block + i / block;
                        let ch = channel(x, y);
                        sum[ch] += data[y * w + x];
                        count[ch] += 1;
                        (sum, count)
                    });
                px.iter_mut()
                    .zip(sum.iter().zip(count))
                    .for_each(|(v, (s, n))| *v = s / n.max(1) as f32);
            }
        });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_bayer(value: f32, w: usize, h: usize) -> Vec<f32> {
        vec![value; w * h]
    }

    #[test]
    fn mhc_reconstructs_flat_image() {
        let w = 16;
        let h = 16;
        let data = flat_bayer(0.5, w, h);
        let out = malvar_he_cutler(&data, w, h, "RGGB");
        for y in 2..h - 2 {
            for x in 2..w - 2 {
                let off = (y * w + x) * 3;
                if (out[off] - 0.5).abs() > 1e-4
                    || (out[off + 1] - 0.5).abs() > 1e-4
                    || (out[off + 2] - 0.5).abs() > 1e-4
                {
                    panic!("non-flat at {x},{y}: {:?}", &out[off..off + 3]);
                }
            }
        }
    }

    #[test]
    fn mhc_matches_size() {
        let w = 8;
        let h = 8;
        let data = flat_bayer(0.3, w, h);
        let out = malvar_he_cutler(&data, w, h, "RGGB");
        if out.len() != w * h * 3 {
            panic!("size mismatch");
        }
    }

    #[test]
    fn mhc_preserves_highlight_headroom() {
        let w = 16;
        let h = 16;
        let data = flat_bayer(2.5, w, h);
        let out = malvar_he_cutler(&data, w, h, "RGGB");
        for y in 2..h - 2 {
            for x in 2..w - 2 {
                let off = (y * w + x) * 3;
                for c in 0..3 {
                    if (out[off + c] - 2.5).abs() > 1e-3 {
                        panic!("clamped headroom at {x},{y} c{c}: {}", out[off + c]);
                    }
                }
            }
        }
    }

    fn mhc_pixel_reference(data: &[f32], w: usize, cfa: &[u8; 4], x: usize, y: usize) -> [f32; 3] {
        let p = |dx: i32, dy: i32| data[(y as i32 + dy) as usize * w + (x as i32 + dx) as usize];
        let own = cfa_channel(cfa, x, y);
        let c = p(0, 0);
        let clamp = |v: f32| v.clamp(0.0, RAW_LINEAR_CEILING);
        let diag = p(-1, -1) + p(1, -1) + p(-1, 1) + p(1, 1);
        let mut rgb = [0.0; 3];
        if own == 1 {
            let n1 = p(-1, 0) + p(1, 0);
            let n2 = p(0, -1) + p(0, 1);
            let d2 = p(-2, 0) + p(2, 0);
            let d2v = p(0, -2) + p(0, 2);
            rgb[cfa_channel(cfa, x + 1, y)] =
                clamp((n1 * 4.0 + c * 5.0 - d2 - diag + d2v * 0.5) / 8.0);
            rgb[cfa_channel(cfa, x, y + 1)] =
                clamp((n2 * 4.0 + c * 5.0 - d2v - diag + d2 * 0.5) / 8.0);
            rgb[1] = c;
        } else {
            let n4 = p(-1, 0) + p(1, 0) + p(0, -1) + p(0, 1);
            let dplus = p(-2, 0) + p(2, 0) + p(0, -2) + p(0, 2);
            rgb[1] = clamp((n4 * 2.0 + c * 4.0 - dplus) / 8.0);
            rgb[2 - own] = clamp((diag * 2.0 + c * 6.0 - dplus * 1.5) / 8.0);
            rgb[own] = c;
        }
        rgb
    }

    #[test]
    fn mhc_rows_match_the_per_pixel_formula_bit_for_bit() {
        let w = 37;
        let h = 29;
        let data: Vec<f32> = (0..w * h)
            .map(|i| ((i as u32).wrapping_mul(2_654_435_761) >> 12) as f32 / 1_048_576.0)
            .collect();
        for pattern in ["RGGB", "BGGR", "GRBG", "GBRG"] {
            let cfa = parse_cfa(pattern);
            let out = malvar_he_cutler(&data, w, h, pattern);
            let interior = (2..h - 2).flat_map(|y| (2..w - 2).map(move |x| (x, y)));
            let border = (0..h)
                .flat_map(|y| (0..w).map(move |x| (x, y)))
                .filter(|&(x, y)| x < 2 || y < 2 || x >= w - 2 || y >= h - 2);
            for (x, y) in interior {
                let got = &out[(y * w + x) * 3..][..3];
                if got != mhc_pixel_reference(&data, w, &cfa, x, y) {
                    panic!("{pattern}: interior pixel {x},{y} differs");
                }
            }
            for (x, y) in border {
                let got = &out[(y * w + x) * 3..][..3];
                if got != bilinear_pixel(&data, w, h, &cfa, x, y) {
                    panic!("{pattern}: border pixel {x},{y} differs");
                }
            }
        }
    }

    const XTRANS: &str = "GGRGGBGGBGGRBRGRBGGGBGGRGGRGGBRBGBRG";

    fn xtrans_reference(data: &[f32], w: usize, h: usize, pattern: &[u8; XTRANS_LEN]) -> Vec<f32> {
        let green: Vec<f32> = (0..w * h)
            .map(|i| {
                let x = i % w;
                let y = i / w;
                if xtrans_channel(pattern, x, y) == 1 {
                    return data[i];
                }
                let (sum, weight) = xtrans_green_edge(data, w, h, pattern, x, y);
                if weight > 0.0 {
                    (sum / weight).clamp(0.0, RAW_LINEAR_CEILING)
                } else {
                    data[i]
                }
            })
            .collect();
        let mut out = vec![0.0; w * h * 3];
        for (i, px) in out.chunks_exact_mut(3).enumerate() {
            let x = i % w;
            let y = i / w;
            let own = xtrans_channel(pattern, x, y);
            px[1] = green[i];
            if own != 1 {
                px[own] = data[i];
            }
            for ch in [0usize, 2].into_iter().filter(|&ch| ch != own) {
                px[ch] = (green[i] + xtrans_chroma(data, &green, (w, h), pattern, (x, y), ch))
                    .clamp(0.0, RAW_LINEAR_CEILING);
            }
        }
        out
    }

    #[test]
    fn xtrans_tap_tables_match_the_per_pixel_formula_bit_for_bit() {
        let w = 29;
        let h = 31;
        let data: Vec<f32> = (0..w * h)
            .map(|i| ((i as u32).wrapping_mul(2_654_435_761) >> 12) as f32 / 1_048_576.0)
            .collect();
        for (sx, sy) in [(0, 0), (1, 0), (3, 2), (5, 5)] {
            let Some(pattern) = parse_xtrans(&shifted_xtrans(sx, sy)) else {
                panic!("shifted pattern {sx},{sy} did not parse");
            };
            if xtrans(&data, w, h, &pattern) != xtrans_reference(&data, w, h, &pattern) {
                panic!("pattern shifted {sx},{sy}: tap tables differ from the per-pixel formula");
            }
        }
    }

    fn xtrans_mosaic(color: [f32; 3], w: usize, h: usize) -> (Vec<f32>, [u8; XTRANS_LEN]) {
        let pattern = parse_xtrans(XTRANS).expect("valid pattern");
        let data = (0..w * h)
            .map(|i| color[xtrans_channel(&pattern, i % w, i / w)])
            .collect();
        (data, pattern)
    }

    fn shifted_xtrans(sx: usize, sy: usize) -> String {
        let pattern = parse_xtrans(XTRANS).expect("valid pattern");
        (0..XTRANS_LEN)
            .map(|i| {
                pattern[((i / XTRANS_DIM + sy) % XTRANS_DIM) * XTRANS_DIM + (i + sx) % XTRANS_DIM]
                    as char
            })
            .collect()
    }

    #[test]
    fn superpixel_recovers_flat_color_for_every_cfa_phase() {
        let color = [0.8f32, 0.5, 0.2];
        let bayer = ["RGGB", "BGGR", "GRBG", "GBRG"].map(|p| (p.to_string(), 2));
        let xtrans = (0..XTRANS_LEN).map(|i| (shifted_xtrans(i % XTRANS_DIM, i / XTRANS_DIM), 3));
        let w = 25;
        let h = 19;
        for (pattern, block) in bayer.into_iter().chain(xtrans) {
            let data: Vec<f32> = match parse_xtrans(&pattern) {
                Some(p) => (0..w * h)
                    .map(|i| color[xtrans_channel(&p, i % w, i / w)])
                    .collect(),
                None => {
                    let cfa = parse_cfa(&pattern);
                    (0..w * h)
                        .map(|i| color[cfa_channel(&cfa, i % w, i / w)])
                        .collect()
                }
            };
            let out = superpixel(&data, w, h, &pattern, block);
            if out.len() != (w / block) * (h / block) * 3 {
                panic!("{pattern}: {} values for {w}x{h} / {block}", out.len());
            }
            if let Some((i, v)) = out
                .iter()
                .enumerate()
                .find(|(i, v)| (**v - color[i % 3]).abs() > 1e-6)
            {
                panic!(
                    "{pattern}: channel {} of pixel {} is {v}, a block lacks that colour",
                    i % 3,
                    i / 3
                );
            }
        }
    }

    #[test]
    fn parse_xtrans_rejects_non_xtrans_patterns() {
        for pattern in ["RGGB", "", "GGRGGBGGBGGRBRGRBGGGBGGRGGRGGBRBGBRE"] {
            if parse_xtrans(pattern).is_some() {
                panic!("accepted {pattern}");
            }
        }
    }

    #[test]
    fn xtrans_reconstructs_constant_color() {
        let w = 24;
        let h = 24;
        let color = [0.8f32, 0.5, 0.2];
        let (data, pattern) = xtrans_mosaic(color, w, h);
        let out = xtrans(&data, w, h, &pattern);
        if out.len() != w * h * 3 {
            panic!("size mismatch");
        }
        for y in 2..h - 2 {
            for x in 2..w - 2 {
                let off = (y * w + x) * 3;
                for c in 0..3 {
                    if (out[off + c] - color[c]).abs() > 1e-4 {
                        panic!("at {x},{y} c{c}: {} want {}", out[off + c], color[c]);
                    }
                }
            }
        }
    }

    #[test]
    fn xtrans_preserves_highlight_headroom() {
        let w = 24;
        let h = 24;
        let (data, pattern) = xtrans_mosaic([2.5, 2.5, 2.5], w, h);
        let out = xtrans(&data, w, h, &pattern);
        for y in 2..h - 2 {
            for x in 2..w - 2 {
                let off = (y * w + x) * 3;
                for c in 0..3 {
                    if (out[off + c] - 2.5).abs() > 1e-3 {
                        panic!("clamped headroom at {x},{y} c{c}: {}", out[off + c]);
                    }
                }
            }
        }
    }
}
