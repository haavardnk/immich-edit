#[inline]
fn cr_weights(t: f32) -> [f32; 4] {
    let t2 = t * t;
    let t3 = t2 * t;
    [
        -0.5 * t3 + t2 - 0.5 * t,
        1.5 * t3 - 2.5 * t2 + 1.0,
        -1.5 * t3 + 2.0 * t2 + 0.5 * t,
        0.5 * t3 - 0.5 * t2,
    ]
}

#[inline]
fn clamp_idx(i: i32, max: i32) -> usize {
    i.clamp(0, max) as usize
}

#[inline]
pub fn sample_channel_bicubic(rgb: &[f32], w: usize, h: usize, x: f32, y: f32, ch: usize) -> f32 {
    let fx = x.floor();
    let fy = y.floor();
    let tx = x - fx;
    let ty = y - fy;
    let wx = cr_weights(tx);
    let wy = cr_weights(ty);
    let max_x = (w as i32) - 1;
    let max_y = (h as i32) - 1;
    let ix0 = fx as i32 - 1;
    let iy0 = fy as i32 - 1;
    let mut sum = 0.0f32;
    for (j, wy_j) in wy.iter().enumerate() {
        let yy = clamp_idx(iy0 + j as i32, max_y);
        let row = yy * w;
        let mut row_sum = 0.0f32;
        for (i, wx_i) in wx.iter().enumerate() {
            let xx = clamp_idx(ix0 + i as i32, max_x);
            row_sum += wx_i * rgb[(row + xx) * 3 + ch];
        }
        sum += wy_j * row_sum;
    }
    sum
}

#[inline]
pub fn sample_rgb_bicubic(rgb: &[f32], w: usize, h: usize, x: f32, y: f32) -> [f32; 3] {
    let fx = x.floor();
    let fy = y.floor();
    let tx = x - fx;
    let ty = y - fy;
    let wx = cr_weights(tx);
    let wy = cr_weights(ty);
    let max_x = (w as i32) - 1;
    let max_y = (h as i32) - 1;
    let ix0 = fx as i32 - 1;
    let iy0 = fy as i32 - 1;
    let mut out = [0.0f32; 3];
    for (j, wy_j) in wy.iter().enumerate() {
        let yy = clamp_idx(iy0 + j as i32, max_y);
        let row = yy * w;
        let mut row_sum = [0.0f32; 3];
        for (i, wx_i) in wx.iter().enumerate() {
            let xx = clamp_idx(ix0 + i as i32, max_x);
            let base = (row + xx) * 3;
            row_sum[0] += wx_i * rgb[base];
            row_sum[1] += wx_i * rgb[base + 1];
            row_sum[2] += wx_i * rgb[base + 2];
        }
        out[0] += wy_j * row_sum[0];
        out[1] += wy_j * row_sum[1];
        out[2] += wy_j * row_sum[2];
    }
    out
}
