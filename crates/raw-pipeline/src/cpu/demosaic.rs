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
    let mut cfa = [b'R', b'G', b'G', b'B'];
    for (i, b) in cfa_pattern.bytes().take(4).enumerate() {
        cfa[i] = b;
    }

    let mut out = vec![0.0f32; w * h * 3];
    out.par_chunks_mut(w * 3).enumerate().for_each(|(y, row)| {
        for x in 0..w {
            let own_ch = cfa_channel(&cfa, x, y);
            let own_val = data[y * w + x];
            let mut rgb = [0.0f32; 3];
            rgb[own_ch] = own_val;

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
                        if cfa_channel(&cfa, nx as usize, ny as usize) == ch {
                            sum += data[ny as usize * w + nx as usize];
                            count += 1;
                        }
                    }
                }
                if count > 0 {
                    *slot = sum / count as f32;
                }
            }

            let off = x * 3;
            row[off] = rgb[0];
            row[off + 1] = rgb[1];
            row[off + 2] = rgb[2];
        }
    });
    out
}

pub fn malvar_he_cutler(data: &[f32], w: usize, h: usize, cfa_pattern: &str) -> Vec<f32> {
    if w < 5 || h < 5 {
        return bilinear(data, w, h, cfa_pattern);
    }
    let mut cfa = [b'R', b'G', b'G', b'B'];
    for (i, b) in cfa_pattern.bytes().take(4).enumerate() {
        cfa[i] = b;
    }

    let mut out = bilinear(data, w, h, cfa_pattern);

    out.par_chunks_mut(w * 3).enumerate().for_each(|(y, row)| {
        if y < 2 || y >= h - 2 {
            return;
        }
        for x in 2..w - 2 {
            let own_ch = cfa_channel(&cfa, x, y);
            let c = data[y * w + x];

            let p = |dx: i32, dy: i32| -> f32 {
                data[((y as i32 + dy) as usize) * w + ((x as i32 + dx) as usize)]
            };

            let row_ch = cfa_channel(&cfa, x + 1, y);
            let col_ch = cfa_channel(&cfa, x, y + 1);

            let off = x * 3;
            if own_ch == 1 {
                let other_h = row_ch;
                let other_v = col_ch;
                let n1 = p(-1, 0) + p(1, 0);
                let n2 = p(0, -1) + p(0, 1);
                let d2 = p(-2, 0) + p(2, 0);
                let d2v = p(0, -2) + p(0, 2);
                let diag = p(-1, -1) + p(1, -1) + p(-1, 1) + p(1, 1);
                let h_val = (n1 * 4.0 + c * 5.0 - d2 - diag + d2v * 0.5) / 8.0;
                let v_val = (n2 * 4.0 + c * 5.0 - d2v - diag + d2 * 0.5) / 8.0;
                row[off + other_h] = h_val.clamp(0.0, 1.0);
                row[off + other_v] = v_val.clamp(0.0, 1.0);
                row[off + 1] = c;
            } else {
                let n4 = p(-1, 0) + p(1, 0) + p(0, -1) + p(0, 1);
                let dplus = p(-2, 0) + p(2, 0) + p(0, -2) + p(0, 2);
                let g_val = (n4 * 2.0 + c * 4.0 - dplus) / 8.0;
                row[off + 1] = g_val.clamp(0.0, 1.0);

                let opp = 2 - own_ch;
                let diag = p(-1, -1) + p(1, -1) + p(-1, 1) + p(1, 1);
                let opp_val =
                    (diag * 2.0 + c * 6.0 - (p(-2, 0) + p(2, 0) + p(0, -2) + p(0, 2)) * 1.5) / 8.0;
                row[off + opp] = opp_val.clamp(0.0, 1.0);
                row[off + own_ch] = c;
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
}
