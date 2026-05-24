use rayon::prelude::*;

pub fn rotate_90(pixels: &[f32], w: usize, h: usize) -> (Vec<f32>, usize, usize) {
    let new_w = h;
    let new_h = w;
    let mut out = vec![0.0f32; new_w * new_h * 3];
    out.par_chunks_mut(new_w * 3)
        .enumerate()
        .for_each(|(dst_y, row)| {
            let x = dst_y;
            for dst_x in 0..new_w {
                let y = h - 1 - dst_x;
                let src = (y * w + x) * 3;
                let d = dst_x * 3;
                row[d] = pixels[src];
                row[d + 1] = pixels[src + 1];
                row[d + 2] = pixels[src + 2];
            }
        });
    (out, new_w, new_h)
}

pub fn apply_orientation(
    rgb: Vec<f32>,
    w: usize,
    h: usize,
    orient: crate::frame::OrientFlips,
) -> (Vec<f32>, usize, usize) {
    let (t, hf, vf) = orient;
    if !t && !hf && !vf {
        return (rgb, w, h);
    }
    let (nw, nh) = if t { (h, w) } else { (w, h) };
    let mut out = vec![0.0f32; nw * nh * 3];
    out.par_chunks_exact_mut(nw * 3)
        .enumerate()
        .for_each(|(oy, row)| {
            for ox in 0..nw {
                let (mut px, mut py) = if t { (oy, ox) } else { (ox, oy) };
                if hf {
                    px = w - 1 - px;
                }
                if vf {
                    py = h - 1 - py;
                }
                let src = (py * w + px) * 3;
                let d = ox * 3;
                row[d] = rgb[src];
                row[d + 1] = rgb[src + 1];
                row[d + 2] = rgb[src + 2];
            }
        });
    (out, nw, nh)
}

pub fn flip_horizontal(pixels: &mut [f32], w: usize, _h: usize) {
    pixels.par_chunks_mut(w * 3).for_each(|row| {
        for x in 0..w / 2 {
            let left = x * 3;
            let right = (w - 1 - x) * 3;
            for c in 0..3 {
                row.swap(left + c, right + c);
            }
        }
    });
}

pub fn flip_vertical(pixels: &mut [f32], w: usize, h: usize) {
    let row_floats = w * 3;
    let half = h / 2;
    if half == 0 {
        return;
    }
    let top_len = half * row_floats;
    let (top, rest) = pixels.split_at_mut(top_len);
    let bottom_offset = if h % 2 == 1 { row_floats } else { 0 };
    let bottom = &mut rest[bottom_offset..];
    top.par_chunks_exact_mut(row_floats)
        .zip(bottom.par_chunks_exact_mut(row_floats).rev())
        .for_each(|(a, b)| a.swap_with_slice(b));
}

pub fn resize_owned(
    pixels: Vec<f32>,
    w: usize,
    h: usize,
    max_edge: u32,
) -> (Vec<f32>, usize, usize) {
    let max = max_edge as usize;
    if w <= max && h <= max {
        return (pixels, w, h);
    }

    let scale = max as f64 / w.max(h) as f64;
    let new_w = (w as f64 * scale).round().max(1.0) as u32;
    let new_h = (h as f64 * scale).round().max(1.0) as u32;

    let mut pixels = pixels;
    match resize_f32x3(&mut pixels, w as u32, h as u32, new_w, new_h) {
        Some(out) => (out, new_w as usize, new_h as usize),
        None => (pixels, w, h),
    }
}

fn resize_f32x3(pixels: &mut [f32], w: u32, h: u32, new_w: u32, new_h: u32) -> Option<Vec<f32>> {
    let src_buf: &mut [u8] = bytemuck::cast_slice_mut(pixels);
    let src_image = fast_image_resize::images::Image::from_slice_u8(
        w,
        h,
        src_buf,
        fast_image_resize::PixelType::F32x3,
    )
    .ok()?;
    let mut dst_image =
        fast_image_resize::images::Image::new(new_w, new_h, fast_image_resize::PixelType::F32x3);
    let mut resizer = fast_image_resize::Resizer::new();
    resizer
        .resize(
            &src_image,
            &mut dst_image,
            Some(&fast_image_resize::ResizeOptions::new().resize_alg(
                fast_image_resize::ResizeAlg::Convolution(fast_image_resize::FilterType::Lanczos3),
            )),
        )
        .ok()?;
    Some(bytemuck::cast_slice(dst_image.buffer()).to_vec())
}
