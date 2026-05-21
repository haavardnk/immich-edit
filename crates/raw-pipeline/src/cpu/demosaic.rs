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
