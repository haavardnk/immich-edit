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
