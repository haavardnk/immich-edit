pub fn rotate_90(pixels: &[f32], w: usize, h: usize) -> (Vec<f32>, usize, usize) {
    let new_w = h;
    let new_h = w;
    let mut out = vec![0.0f32; new_w * new_h * 3];
    for y in 0..h {
        for x in 0..w {
            let src = (y * w + x) * 3;
            let dst_x = h - 1 - y;
            let dst_y = x;
            let dst = (dst_y * new_w + dst_x) * 3;
            out[dst] = pixels[src];
            out[dst + 1] = pixels[src + 1];
            out[dst + 2] = pixels[src + 2];
        }
    }
    (out, new_w, new_h)
}

pub fn transpose(pixels: &[f32], w: usize, h: usize) -> (Vec<f32>, usize, usize) {
    let new_w = h;
    let new_h = w;
    let mut out = vec![0.0f32; new_w * new_h * 3];
    for y in 0..h {
        for x in 0..w {
            let src = (y * w + x) * 3;
            let dst = (x * new_w + y) * 3;
            out[dst] = pixels[src];
            out[dst + 1] = pixels[src + 1];
            out[dst + 2] = pixels[src + 2];
        }
    }
    (out, new_w, new_h)
}

pub fn apply_orientation(
    rgb: Vec<f32>,
    w: usize,
    h: usize,
    orient: crate::frame::OrientFlips,
) -> (Vec<f32>, usize, usize) {
    let (t, hf, vf) = orient;
    let mut rgb = rgb;
    let mut w = w;
    let mut h = h;
    if hf {
        flip_horizontal(&mut rgb, w, h);
    }
    if vf {
        flip_vertical(&mut rgb, w, h);
    }
    if t {
        let (r, nw, nh) = transpose(&rgb, w, h);
        rgb = r;
        w = nw;
        h = nh;
    }
    (rgb, w, h)
}

pub fn flip_horizontal(pixels: &mut [f32], w: usize, h: usize) {
    for y in 0..h {
        for x in 0..w / 2 {
            let left = (y * w + x) * 3;
            let right = (y * w + (w - 1 - x)) * 3;
            for c in 0..3 {
                pixels.swap(left + c, right + c);
            }
        }
    }
}

pub fn flip_vertical(pixels: &mut [f32], w: usize, h: usize) {
    for y in 0..h / 2 {
        for x in 0..w {
            let top = (y * w + x) * 3;
            let bot = ((h - 1 - y) * w + x) * 3;
            for c in 0..3 {
                pixels.swap(top + c, bot + c);
            }
        }
    }
}

pub fn crop(
    pixels: &[f32],
    w: usize,
    h: usize,
    x: f64,
    y: f64,
    cw: f64,
    ch: f64,
) -> (Vec<f32>, usize, usize) {
    let sx = (x * w as f64) as usize;
    let sy = (y * h as f64) as usize;
    let sw = ((cw * w as f64) as usize).max(1).min(w - sx);
    let sh = ((ch * h as f64) as usize).max(1).min(h - sy);

    let mut out = vec![0.0f32; sw * sh * 3];
    for row in 0..sh {
        let src_start = ((sy + row) * w + sx) * 3;
        let dst_start = row * sw * 3;
        out[dst_start..dst_start + sw * 3].copy_from_slice(&pixels[src_start..src_start + sw * 3]);
    }
    (out, sw, sh)
}

pub fn resize(pixels: &[f32], w: usize, h: usize, max_edge: u32) -> (Vec<f32>, usize, usize) {
    let max = max_edge as usize;
    if w <= max && h <= max {
        return (pixels.to_vec(), w, h);
    }

    let scale = max as f64 / w.max(h) as f64;
    let new_w = (w as f64 * scale).round() as usize;
    let new_h = (h as f64 * scale).round() as usize;
    let new_w = new_w.max(1);
    let new_h = new_h.max(1);

    let mut out = vec![0.0f32; new_w * new_h * 3];
    for ny in 0..new_h {
        let sy = (ny as f64 / new_h as f64 * h as f64).min((h - 1) as f64);
        let sy0 = sy as usize;
        let sy1 = (sy0 + 1).min(h - 1);
        let fy = sy - sy0 as f64;

        for nx in 0..new_w {
            let sx = (nx as f64 / new_w as f64 * w as f64).min((w - 1) as f64);
            let sx0 = sx as usize;
            let sx1 = (sx0 + 1).min(w - 1);
            let fx = sx - sx0 as f64;

            for c in 0..3 {
                let p00 = pixels[(sy0 * w + sx0) * 3 + c] as f64;
                let p10 = pixels[(sy0 * w + sx1) * 3 + c] as f64;
                let p01 = pixels[(sy1 * w + sx0) * 3 + c] as f64;
                let p11 = pixels[(sy1 * w + sx1) * 3 + c] as f64;
                let v = p00 * (1.0 - fx) * (1.0 - fy)
                    + p10 * fx * (1.0 - fy)
                    + p01 * (1.0 - fx) * fy
                    + p11 * fx * fy;
                out[(ny * new_w + nx) * 3 + c] = v as f32;
            }
        }
    }
    (out, new_w, new_h)
}
