use crate::model::{Activation, Fit, Layout, ModelSpec};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placement {
    pub edge: usize,
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

impl Placement {
    pub fn compute(fit: Fit, edge: usize, src_w: u32, src_h: u32) -> Self {
        match fit {
            Fit::Stretch => Self {
                edge,
                x: 0,
                y: 0,
                w: edge,
                h: edge,
            },
            Fit::Contain => {
                let long = src_w.max(src_h).max(1) as f32;
                let scale = edge as f32 / long;
                let w = ((src_w as f32 * scale).round() as usize).clamp(1, edge);
                let h = ((src_h as f32 * scale).round() as usize).clamp(1, edge);
                Self {
                    edge,
                    x: (edge - w) / 2,
                    y: (edge - h) / 2,
                    w,
                    h,
                }
            }
        }
    }
}

pub fn to_tensor(rgb8: &[u8], w: u32, h: u32, spec: &ModelSpec) -> (Vec<f32>, Placement) {
    let edge = spec.input_edge as usize;
    let place = Placement::compute(spec.fit, edge, w, h);
    let n = spec.normalization;
    let mut out = vec![0.0f32; edge * edge * 3];
    let plane = edge * edge;
    let sx = w as f32 / place.w as f32;
    let sy = h as f32 / place.h as f32;
    for y in 0..place.h {
        let fy = ((y as f32 + 0.5) * sy - 0.5).max(0.0);
        let y0 = (fy.floor() as usize).min(h as usize - 1);
        let y1 = (y0 + 1).min(h as usize - 1);
        let ty = fy - y0 as f32;
        for x in 0..place.w {
            let fx = ((x as f32 + 0.5) * sx - 0.5).max(0.0);
            let x0 = (fx.floor() as usize).min(w as usize - 1);
            let x1 = (x0 + 1).min(w as usize - 1);
            let tx = fx - x0 as f32;
            let i00 = (y0 * w as usize + x0) * 3;
            let i01 = (y0 * w as usize + x1) * 3;
            let i10 = (y1 * w as usize + x0) * 3;
            let i11 = (y1 * w as usize + x1) * 3;
            let dx = place.x + x;
            let dy = place.y + y;
            for c in 0..3 {
                let top = rgb8[i00 + c] as f32 + (rgb8[i01 + c] as f32 - rgb8[i00 + c] as f32) * tx;
                let bot = rgb8[i10 + c] as f32 + (rgb8[i11 + c] as f32 - rgb8[i10 + c] as f32) * tx;
                let v = (top + (bot - top) * ty) * n.scale;
                let v = (v - n.mean[c]) / n.std[c];
                let idx = match spec.layout {
                    Layout::Nchw => c * plane + dy * edge + dx,
                    Layout::Nhwc => (dy * edge + dx) * 3 + c,
                };
                out[idx] = v;
            }
        }
    }
    (out, place)
}

pub fn mask_to_source(
    mask: &[f32],
    mask_w: usize,
    mask_h: usize,
    place: &Placement,
    out_w: usize,
    out_h: usize,
) -> Vec<f32> {
    let scale_x = mask_w as f32 / place.edge as f32;
    let scale_y = mask_h as f32 / place.edge as f32;
    let x0 = ((place.x as f32 * scale_x).round() as usize).min(mask_w - 1);
    let y0 = ((place.y as f32 * scale_y).round() as usize).min(mask_h - 1);
    let cw = ((place.w as f32 * scale_x).round() as usize).clamp(1, mask_w - x0);
    let ch = ((place.h as f32 * scale_y).round() as usize).clamp(1, mask_h - y0);
    let mut cropped = vec![0.0f32; cw * ch];
    for y in 0..ch {
        for x in 0..cw {
            cropped[y * cw + x] = mask[(y0 + y) * mask_w + (x0 + x)];
        }
    }
    resize_mask(&cropped, cw, ch, out_w, out_h)
}

pub fn activate(values: &[f32], w: usize, h: usize, activation: Activation) -> Vec<f32> {
    match activation {
        Activation::Sigmoid => values.iter().map(|v| 1.0 / (1.0 + (-v).exp())).collect(),
        Activation::None => values.to_vec(),
        Activation::MinMax => {
            let min = values.iter().copied().fold(f32::INFINITY, f32::min);
            let max = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            let span = (max - min).max(1e-6);
            values.iter().map(|v| (v - min) / span).collect()
        }
        Activation::Softmax { channel } => {
            let plane = w * h;
            let classes = values.len() / plane.max(1);
            let mut out = vec![0.0f32; plane];
            if plane == 0 || channel >= classes {
                return out;
            }
            for i in 0..plane {
                let mut max = f32::NEG_INFINITY;
                for c in 0..classes {
                    max = max.max(values[c * plane + i]);
                }
                let mut sum = 0.0f32;
                for c in 0..classes {
                    sum += (values[c * plane + i] - max).exp();
                }
                out[i] = (values[channel * plane + i] - max).exp() / sum.max(1e-12);
            }
            out
        }
    }
}

pub fn resize_mask(mask: &[f32], w: usize, h: usize, out_w: usize, out_h: usize) -> Vec<f32> {
    if w == out_w && h == out_h {
        return mask.to_vec();
    }
    let mut out = vec![0.0f32; out_w * out_h];
    let sx = w as f32 / out_w as f32;
    let sy = h as f32 / out_h as f32;
    for y in 0..out_h {
        let fy = ((y as f32 + 0.5) * sy - 0.5).max(0.0);
        let y0 = (fy.floor() as usize).min(h - 1);
        let y1 = (y0 + 1).min(h - 1);
        let ty = fy - y0 as f32;
        for x in 0..out_w {
            let fx = ((x as f32 + 0.5) * sx - 0.5).max(0.0);
            let x0 = (fx.floor() as usize).min(w - 1);
            let x1 = (x0 + 1).min(w - 1);
            let tx = fx - x0 as f32;
            let top = mask[y0 * w + x0] + (mask[y0 * w + x1] - mask[y0 * w + x0]) * tx;
            let bot = mask[y1 * w + x0] + (mask[y1 * w + x1] - mask[y1 * w + x0]) * tx;
            out[y * out_w + x] = top + (bot - top) * ty;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ModelKind, Normalization};

    fn spec(edge: u32, layout: Layout) -> ModelSpec {
        ModelSpec {
            kind: ModelKind::Subject,
            input_edge: edge,
            layout,
            fit: Fit::Stretch,
            normalization: Normalization::UNIT,
            activation: Activation::Sigmoid,
        }
    }

    #[test]
    fn tensor_layout_and_normalization() {
        let rgb = vec![255u8, 0, 0, 255, 0, 0, 255, 0, 0, 255, 0, 0];
        let (nchw, _) = to_tensor(&rgb, 2, 2, &spec(2, Layout::Nchw));
        assert_eq!(nchw.len(), 12);
        assert!((nchw[0] - 1.0).abs() < 1e-6);
        assert!(nchw[4].abs() < 1e-6);
        let (nhwc, _) = to_tensor(&rgb, 2, 2, &spec(2, Layout::Nhwc));
        assert!((nhwc[0] - 1.0).abs() < 1e-6);
        assert!(nhwc[1].abs() < 1e-6);
    }

    #[test]
    fn imagenet_normalization_matches_reference() {
        let rgb = vec![128u8, 128, 128];
        let mut s = spec(1, Layout::Nchw);
        s.normalization = Normalization::IMAGENET;
        let (t, _) = to_tensor(&rgb, 1, 1, &s);
        let expect = (128.0 / 255.0 - 0.485) / 0.229;
        assert!((t[0] - expect).abs() < 1e-5);
    }

    #[test]
    fn stretch_fills_the_square() {
        let p = Placement::compute(Fit::Stretch, 64, 100, 200);
        assert_eq!((p.x, p.y, p.w, p.h), (0, 0, 64, 64));
    }

    #[test]
    fn contain_preserves_aspect_and_centres() {
        let p = Placement::compute(Fit::Contain, 64, 100, 200);
        assert_eq!(p.h, 64);
        assert_eq!(p.w, 32);
        assert_eq!(p.y, 0);
        assert_eq!(p.x, 16);
    }

    #[test]
    fn contain_leaves_padding_untouched() {
        let mut s = spec(4, Layout::Nchw);
        s.fit = Fit::Contain;
        let rgb = vec![255u8; 2 * 4 * 3];
        let (t, p) = to_tensor(&rgb, 2, 4, &s);
        assert_eq!((p.x, p.w), (1, 2));
        assert!(t[0].abs() < 1e-6);
        assert!((t[p.y * 4 + p.x] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn mask_to_source_crops_the_padded_region() {
        let place = Placement {
            edge: 4,
            x: 1,
            y: 0,
            w: 2,
            h: 4,
        };
        let mut mask = vec![0.0f32; 16];
        for y in 0..4 {
            for x in 1..3 {
                mask[y * 4 + x] = 1.0;
            }
        }
        let out = mask_to_source(&mask, 4, 4, &place, 2, 4);
        assert_eq!(out.len(), 8);
        assert!(out.iter().all(|v| (*v - 1.0).abs() < 1e-6));
    }

    #[test]
    fn softmax_selects_channel() {
        let values = vec![0.0, 0.0, 2.0, 2.0, 0.0, 0.0];
        let out = activate(&values, 2, 1, Activation::Softmax { channel: 1 });
        let expect = 2.0f32.exp() / (2.0 + 2.0f32.exp());
        assert_eq!(out.len(), 2);
        assert!((out[0] - expect).abs() < 1e-5);
        assert!((out[1] - expect).abs() < 1e-5);
    }

    #[test]
    fn softmax_out_of_range_channel_is_empty_not_a_panic() {
        let values = vec![0.0, 0.0, 2.0, 2.0, 0.0, 0.0];
        let out = activate(&values, 2, 1, Activation::Softmax { channel: 9 });
        assert_eq!(out.len(), 2);
        assert!(out.iter().all(|v| *v == 0.0));
    }

    #[test]
    fn minmax_spans_unit_range() {
        let out = activate(&[-1.0, 0.0, 3.0], 3, 1, Activation::MinMax);
        assert!(out[0].abs() < 1e-6);
        assert!((out[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn resize_mask_is_identity_and_interpolates() {
        let m = vec![0.0, 1.0, 0.0, 1.0];
        assert_eq!(resize_mask(&m, 2, 2, 2, 2), m);
        let up = resize_mask(&m, 2, 2, 4, 4);
        assert_eq!(up.len(), 16);
        assert!(up[0] <= up[3]);
    }
}
