use crate::tone::shared::RAW_LINEAR_CEILING;
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
    out.par_chunks_exact_mut(w * 3)
        .enumerate()
        .for_each(|(y, row)| {
            for x in 0..w {
                let off = x * 3;
                if y < 2 || y >= h - 2 || x < 2 || x >= w - 2 {
                    let rgb = bilinear_pixel(data, w, h, &cfa, x, y);
                    row[off] = rgb[0];
                    row[off + 1] = rgb[1];
                    row[off + 2] = rgb[2];
                    continue;
                }
                let own_ch = cfa_channel(&cfa, x, y);
                let c = data[y * w + x];
                let p = |dx: i32, dy: i32| -> f32 {
                    data[((y as i32 + dy) as usize) * w + ((x as i32 + dx) as usize)]
                };
                let row_ch = cfa_channel(&cfa, x + 1, y);
                let col_ch = cfa_channel(&cfa, x, y + 1);
                if own_ch == 1 {
                    let n1 = p(-1, 0) + p(1, 0);
                    let n2 = p(0, -1) + p(0, 1);
                    let d2 = p(-2, 0) + p(2, 0);
                    let d2v = p(0, -2) + p(0, 2);
                    let diag = p(-1, -1) + p(1, -1) + p(-1, 1) + p(1, 1);
                    let h_val = (n1 * 4.0 + c * 5.0 - d2 - diag + d2v * 0.5) / 8.0;
                    let v_val = (n2 * 4.0 + c * 5.0 - d2v - diag + d2 * 0.5) / 8.0;
                    row[off + row_ch] = h_val.clamp(0.0, RAW_LINEAR_CEILING);
                    row[off + col_ch] = v_val.clamp(0.0, RAW_LINEAR_CEILING);
                    row[off + 1] = c;
                } else {
                    let n4 = p(-1, 0) + p(1, 0) + p(0, -1) + p(0, 1);
                    let dplus = p(-2, 0) + p(2, 0) + p(0, -2) + p(0, 2);
                    let g_val = (n4 * 2.0 + c * 4.0 - dplus) / 8.0;
                    row[off + 1] = g_val.clamp(0.0, RAW_LINEAR_CEILING);
                    let opp = 2 - own_ch;
                    let diag = p(-1, -1) + p(1, -1) + p(-1, 1) + p(1, 1);
                    let opp_val = (diag * 2.0 + c * 6.0
                        - (p(-2, 0) + p(2, 0) + p(0, -2) + p(0, 2)) * 1.5)
                        / 8.0;
                    row[off + opp] = opp_val.clamp(0.0, RAW_LINEAR_CEILING);
                    row[off + own_ch] = c;
                }
            }
        });
    out
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

pub fn shift_xtrans(pattern: &[u8; XTRANS_LEN], dx: usize, dy: usize) -> [u8; XTRANS_LEN] {
    let mut out = [b'G'; XTRANS_LEN];
    for (i, slot) in out.iter_mut().enumerate() {
        let x = (i % XTRANS_DIM + dx) % XTRANS_DIM;
        let y = (i / XTRANS_DIM + dy) % XTRANS_DIM;
        *slot = pattern[y * XTRANS_DIM + x];
    }
    out
}

fn xtrans_channel(pattern: &[u8; XTRANS_LEN], x: usize, y: usize) -> usize {
    match pattern[(y % XTRANS_DIM) * XTRANS_DIM + x % XTRANS_DIM] {
        b'R' => 0,
        b'B' => 2,
        _ => 1,
    }
}

fn xtrans_green(data: &[f32], w: usize, h: usize, pattern: &[u8; XTRANS_LEN]) -> Vec<f32> {
    let mut green = vec![0.0f32; w * h];
    green.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
        for (x, slot) in row.iter_mut().enumerate() {
            if xtrans_channel(pattern, x, y) == 1 {
                *slot = data[y * w + x];
                continue;
            }
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
            *slot = if weight > 0.0 {
                (sum / weight).clamp(0.0, RAW_LINEAR_CEILING)
            } else {
                data[y * w + x]
            };
        }
    });
    green
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
    let mut out = vec![0.0f32; w * h * 3];
    out.par_chunks_mut(w * 3).enumerate().for_each(|(y, row)| {
        for x in 0..w {
            let i = y * w + x;
            let off = x * 3;
            let own = xtrans_channel(pattern, x, y);
            row[off + 1] = green[i];
            if own != 1 {
                row[off + own] = data[i];
            }
            for ch in [0usize, 2] {
                if ch == own {
                    continue;
                }
                row[off + ch] = (green[i]
                    + xtrans_chroma(data, &green, (w, h), pattern, (x, y), ch))
                .clamp(0.0, RAW_LINEAR_CEILING);
            }
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

    const XTRANS: &str = "GGRGGBGGBGGRBRGRBGGGBGGRGGRGGBRBGBRG";

    fn xtrans_mosaic(color: [f32; 3], w: usize, h: usize) -> (Vec<f32>, [u8; XTRANS_LEN]) {
        let pattern = parse_xtrans(XTRANS).expect("valid pattern");
        let data = (0..w * h)
            .map(|i| color[xtrans_channel(&pattern, i % w, i / w)])
            .collect();
        (data, pattern)
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
    fn shift_xtrans_matches_offset_lookup() {
        let pattern = parse_xtrans(XTRANS).expect("valid pattern");
        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (2, 3), (7, 11)] {
            let shifted = shift_xtrans(&pattern, dx, dy);
            for i in 0..XTRANS_LEN {
                let x = i % XTRANS_DIM;
                let y = i / XTRANS_DIM;
                if xtrans_channel(&shifted, x, y) != xtrans_channel(&pattern, x + dx, y + dy) {
                    panic!("shift {dx},{dy} wrong at {x},{y}");
                }
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
