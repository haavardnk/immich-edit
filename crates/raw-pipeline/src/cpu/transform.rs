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

pub fn transpose(pixels: &[f32], w: usize, h: usize) -> (Vec<f32>, usize, usize) {
    let new_w = h;
    let new_h = w;
    let mut out = vec![0.0f32; new_w * new_h * 3];
    out.par_chunks_mut(new_w * 3)
        .enumerate()
        .for_each(|(dst_y, row)| {
            let x = dst_y;
            for dst_x in 0..new_w {
                let src = (dst_x * w + x) * 3;
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
    let row_bytes = w * 3;
    let mut out = vec![0.0f32; pixels.len()];
    out.par_chunks_mut(row_bytes)
        .enumerate()
        .for_each(|(y, row)| {
            let src_y = h - 1 - y;
            let src_start = src_y * row_bytes;
            row.copy_from_slice(&pixels[src_start..src_start + row_bytes]);
        });
    pixels.copy_from_slice(&out);
}

pub fn resize(pixels: &[f32], w: usize, h: usize, max_edge: u32) -> (Vec<f32>, usize, usize) {
    let max = max_edge as usize;
    if w <= max && h <= max {
        return (pixels.to_vec(), w, h);
    }

    let scale = max as f64 / w.max(h) as f64;
    let new_w = (w as f64 * scale).round().max(1.0) as u32;
    let new_h = (h as f64 * scale).round().max(1.0) as u32;

    let mut src_buf = bytemuck::cast_slice::<f32, u8>(pixels).to_vec();
    let src_image = match fast_image_resize::images::Image::from_slice_u8(
        w as u32,
        h as u32,
        &mut src_buf,
        fast_image_resize::PixelType::F32x3,
    ) {
        Ok(img) => img,
        Err(_) => return (pixels.to_vec(), w, h),
    };

    let mut dst_image =
        fast_image_resize::images::Image::new(new_w, new_h, fast_image_resize::PixelType::F32x3);

    let mut resizer = fast_image_resize::Resizer::new();
    if resizer
        .resize(
            &src_image,
            &mut dst_image,
            Some(&fast_image_resize::ResizeOptions::new().resize_alg(
                fast_image_resize::ResizeAlg::Convolution(fast_image_resize::FilterType::Lanczos3),
            )),
        )
        .is_err()
    {
        return (pixels.to_vec(), w, h);
    }

    let out: Vec<f32> = bytemuck::cast_slice(dst_image.buffer()).to_vec();
    (out, new_w as usize, new_h as usize)
}
