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

struct Taps {
    rows: [usize; 4],
    cols: [usize; 4],
    wx: [f32; 4],
    wy: [f32; 4],
}

impl Taps {
    #[inline(always)]
    fn new(w: usize, h: usize, x: f32, y: f32) -> Self {
        let fx = x.floor();
        let fy = y.floor();
        let ix0 = fx as i32 - 1;
        let iy0 = fy as i32 - 1;
        let max_x = w as i32 - 1;
        let max_y = h as i32 - 1;
        Self {
            rows: std::array::from_fn(|j| (iy0 + j as i32).clamp(0, max_y) as usize * w * 3),
            cols: std::array::from_fn(|i| (ix0 + i as i32).clamp(0, max_x) as usize * 3),
            wx: cr_weights(x - fx),
            wy: cr_weights(y - fy),
        }
    }

    #[inline(always)]
    fn contiguous(&self) -> bool {
        self.cols[3] == self.cols[0] + 9
    }
}

#[inline]
pub fn sample_channel_bicubic(rgb: &[f32], w: usize, h: usize, x: f32, y: f32, ch: usize) -> f32 {
    let t = Taps::new(w, h, x, y);
    let mut sum = 0.0f32;
    if t.contiguous() && ch < 3 {
        for (row, wy) in t.rows.into_iter().zip(t.wy) {
            let start = row + t.cols[0] + ch;
            let px = &rgb[start..start + 10];
            sum += wy * (t.wx[0] * px[0] + t.wx[1] * px[3] + t.wx[2] * px[6] + t.wx[3] * px[9]);
        }
        return sum;
    }
    for (row, wy) in t.rows.into_iter().zip(t.wy) {
        let mut row_sum = 0.0f32;
        for (col, wx) in t.cols.into_iter().zip(t.wx) {
            row_sum += wx * rgb[row + col + ch];
        }
        sum += wy * row_sum;
    }
    sum
}

#[inline]
pub fn sample_rgb_bicubic(rgb: &[f32], w: usize, h: usize, x: f32, y: f32) -> [f32; 3] {
    let t = Taps::new(w, h, x, y);
    let mut out = [0.0f32; 3];
    if t.contiguous() {
        for (row, wy) in t.rows.into_iter().zip(t.wy) {
            let start = row + t.cols[0];
            let px = &rgb[start..start + 12];
            for (c, o) in out.iter_mut().enumerate() {
                *o += wy
                    * (t.wx[0] * px[c]
                        + t.wx[1] * px[3 + c]
                        + t.wx[2] * px[6 + c]
                        + t.wx[3] * px[9 + c]);
            }
        }
        return out;
    }
    for (row, wy) in t.rows.into_iter().zip(t.wy) {
        let mut row_sum = [0.0f32; 3];
        for (col, wx) in t.cols.into_iter().zip(t.wx) {
            let px = &rgb[row + col..row + col + 3];
            row_sum[0] += wx * px[0];
            row_sum[1] += wx * px[1];
            row_sum[2] += wx * px[2];
        }
        out[0] += wy * row_sum[0];
        out[1] += wy * row_sum[1];
        out[2] += wy * row_sum[2];
    }
    out
}
