use crate::frame::RawFrame;

pub fn demosaic(frame: &RawFrame) -> Vec<f32> {
    if frame.cpp == 3 {
        return frame.data.clone();
    }

    let w = frame.width;
    let h = frame.height;
    let mut rgb = vec![0.0f32; w * h * 3];
    let cfa = &frame.cfa_pattern;

    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let val = frame.data[idx];
            let (cfa_x, cfa_y) = (x % 2, y % 2);
            let color = cfa_color(cfa, cfa_x, cfa_y);

            match color {
                CfaColor::Red => {
                    rgb[idx * 3] = val;
                    rgb[idx * 3 + 1] = interpolate_green(frame, x, y);
                    rgb[idx * 3 + 2] = interpolate_cross(frame, x, y);
                }
                CfaColor::Green => {
                    rgb[idx * 3] = interpolate_hv(frame, x, y, cfa, true);
                    rgb[idx * 3 + 1] = val;
                    rgb[idx * 3 + 2] = interpolate_hv(frame, x, y, cfa, false);
                }
                CfaColor::Blue => {
                    rgb[idx * 3] = interpolate_cross(frame, x, y);
                    rgb[idx * 3 + 1] = interpolate_green(frame, x, y);
                    rgb[idx * 3 + 2] = val;
                }
            }
        }
    }

    rgb
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CfaColor {
    Red,
    Green,
    Blue,
}

fn cfa_color(pattern: &str, x: usize, y: usize) -> CfaColor {
    let chars: Vec<char> = pattern.chars().collect();
    let idx = y * 2 + x;
    if idx >= chars.len() {
        return CfaColor::Green;
    }
    match chars[idx] {
        'R' => CfaColor::Red,
        'G' => CfaColor::Green,
        'B' => CfaColor::Blue,
        _ => CfaColor::Green,
    }
}

fn sample(frame: &RawFrame, x: isize, y: isize) -> f32 {
    let x = x.clamp(0, frame.width as isize - 1) as usize;
    let y = y.clamp(0, frame.height as isize - 1) as usize;
    frame.data[y * frame.width + x]
}

fn interpolate_green(frame: &RawFrame, x: usize, y: usize) -> f32 {
    let (ix, iy) = (x as isize, y as isize);
    let sum = sample(frame, ix - 1, iy)
        + sample(frame, ix + 1, iy)
        + sample(frame, ix, iy - 1)
        + sample(frame, ix, iy + 1);
    sum / 4.0
}

fn interpolate_cross(frame: &RawFrame, x: usize, y: usize) -> f32 {
    let (ix, iy) = (x as isize, y as isize);
    let sum = sample(frame, ix - 1, iy - 1)
        + sample(frame, ix + 1, iy - 1)
        + sample(frame, ix - 1, iy + 1)
        + sample(frame, ix + 1, iy + 1);
    sum / 4.0
}

fn interpolate_hv(frame: &RawFrame, x: usize, y: usize, cfa: &str, want_red: bool) -> f32 {
    let (ix, iy) = (x as isize, y as isize);
    let (cfa_x, cfa_y) = (x % 2, y % 2);

    let above = cfa_color(cfa, cfa_x, (cfa_y + 1) % 2);
    let left = cfa_color(cfa, (cfa_x + 1) % 2, cfa_y);

    let target = if want_red { CfaColor::Red } else { CfaColor::Blue };

    if above == target {
        (sample(frame, ix, iy - 1) + sample(frame, ix, iy + 1)) / 2.0
    } else if left == target {
        (sample(frame, ix - 1, iy) + sample(frame, ix + 1, iy)) / 2.0
    } else {
        (sample(frame, ix - 1, iy - 1)
            + sample(frame, ix + 1, iy - 1)
            + sample(frame, ix - 1, iy + 1)
            + sample(frame, ix + 1, iy + 1))
            / 4.0
    }
}
