use std::path::Path;

use ort::value::TensorRef;

use crate::runtime::{Backend, RuntimeMode, SegmentError, SegmentRuntime, SessionConfig, ort_err};
use crate::segmenter::Mask;

pub const SAM_EDGE: usize = 1024;
const SAM_MEAN: [f32; 3] = [123.675, 116.28, 103.53];
const SAM_STD: [f32; 3] = [58.395, 57.12, 57.375];
const LOW_RES: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClickPoint {
    pub x: f32,
    pub y: f32,
    pub positive: bool,
}

#[derive(Debug, Clone)]
pub struct Embedding {
    pub values: Vec<f32>,
    pub dims: Vec<i64>,
    pub scale: f32,
    pub width: u32,
    pub height: u32,
}

pub struct SamEncoder {
    runtime: SegmentRuntime,
}

pub struct SamDecoder {
    runtime: SegmentRuntime,
}

impl SamEncoder {
    pub fn open(
        path: &Path,
        mode: RuntimeMode,
        config: &SessionConfig,
    ) -> Result<Self, SegmentError> {
        Ok(Self {
            runtime: SegmentRuntime::open(path, mode, config)?,
        })
    }

    pub fn backend(&self) -> Backend {
        self.runtime.backend
    }

    pub fn encode(
        &mut self,
        rgb8: &[u8],
        width: u32,
        height: u32,
    ) -> Result<Embedding, SegmentError> {
        if rgb8.len() != (width as usize) * (height as usize) * 3 {
            return Err(SegmentError::Input(
                "rgb8 length does not match dimensions".into(),
            ));
        }
        let (pixels, scale) = letterbox(rgb8, width, height);
        let plane = SAM_EDGE * SAM_EDGE;
        let mut chw = vec![0f32; plane * 3];
        for i in 0..plane {
            for c in 0..3 {
                chw[c * plane + i] = (pixels[i * 3 + c] as f32 - SAM_MEAN[c]) / SAM_STD[c];
            }
        }

        let input = self.runtime.session.inputs()[0].name().to_string();
        let output = self.runtime.session.outputs()[0].name().to_string();
        let outputs = self
            .runtime
            .session
            .run(ort::inputs![
                input.as_str() => TensorRef::from_array_view(
                    (vec![1usize, 3, SAM_EDGE, SAM_EDGE], chw.as_slice()),
                ).map_err(ort_err)?
            ])
            .map_err(ort_err)?;

        let (dims, values) = outputs[output.as_str()]
            .try_extract_tensor::<f32>()
            .map_err(ort_err)?;
        Ok(Embedding {
            values: values.to_vec(),
            dims: dims.to_vec(),
            scale,
            width,
            height,
        })
    }
}

impl SamDecoder {
    pub fn open(
        path: &Path,
        mode: RuntimeMode,
        config: &SessionConfig,
    ) -> Result<Self, SegmentError> {
        Ok(Self {
            runtime: SegmentRuntime::open(path, mode, config)?,
        })
    }

    pub fn backend(&self) -> Backend {
        self.runtime.backend
    }

    pub fn decode(
        &mut self,
        embedding: &Embedding,
        points: &[ClickPoint],
    ) -> Result<Mask, SegmentError> {
        if points.is_empty() {
            return Err(SegmentError::Input("no points given".into()));
        }
        let mut coords: Vec<f32> = Vec::with_capacity(points.len() * 2);
        let mut labels: Vec<f32> = Vec::with_capacity(points.len());
        for p in points {
            coords.push(p.x * embedding.scale);
            coords.push(p.y * embedding.scale);
            labels.push(if p.positive { 1.0 } else { 0.0 });
        }
        let mask_input = vec![0f32; LOW_RES * LOW_RES];
        let has_mask = vec![0f32; 1];
        let orig_size = vec![embedding.height as f32, embedding.width as f32];
        let emb_shape: Vec<usize> = embedding.dims.iter().map(|d| *d as usize).collect();

        let names: Vec<String> = self
            .runtime
            .session
            .inputs()
            .iter()
            .map(|i| i.name().to_string())
            .collect();
        if names.len() < 6 {
            return Err(SegmentError::Input(format!(
                "decoder expects 6 inputs, model has {}",
                names.len()
            )));
        }
        let output = self.runtime.session.outputs()[0].name().to_string();

        let outputs = self
            .runtime
            .session
            .run(ort::inputs![
                names[0].as_str() => TensorRef::from_array_view((emb_shape, embedding.values.as_slice())).map_err(ort_err)?,
                names[1].as_str() => TensorRef::from_array_view((vec![1usize, points.len(), 2], coords.as_slice())).map_err(ort_err)?,
                names[2].as_str() => TensorRef::from_array_view((vec![1usize, points.len()], labels.as_slice())).map_err(ort_err)?,
                names[3].as_str() => TensorRef::from_array_view((vec![1usize, 1, LOW_RES, LOW_RES], mask_input.as_slice())).map_err(ort_err)?,
                names[4].as_str() => TensorRef::from_array_view((vec![1usize], has_mask.as_slice())).map_err(ort_err)?,
                names[5].as_str() => TensorRef::from_array_view((vec![2usize], orig_size.as_slice())).map_err(ort_err)?,
            ])
            .map_err(ort_err)?;

        let (dims, values) = outputs[output.as_str()]
            .try_extract_tensor::<f32>()
            .map_err(ort_err)?;
        if dims.len() < 2 {
            return Err(SegmentError::Input("decoder output has no plane".into()));
        }
        let h = dims[dims.len() - 2] as usize;
        let w = dims[dims.len() - 1] as usize;
        let plane = w * h;
        if values.len() < plane {
            return Err(SegmentError::Input(format!(
                "decoder returned {} values, expected {plane}",
                values.len()
            )));
        }
        Ok(Mask {
            values: values[..plane].iter().map(|v| sigmoid(*v)).collect(),
            width: w,
            height: h,
        })
    }
}

fn sigmoid(v: f32) -> f32 {
    1.0 / (1.0 + (-v).exp())
}

fn letterbox(rgb8: &[u8], width: u32, height: u32) -> (Vec<u8>, f32) {
    let scale = SAM_EDGE as f32 / width.max(height) as f32;
    let nw = ((width as f32 * scale).round() as usize).min(SAM_EDGE);
    let nh = ((height as f32 * scale).round() as usize).min(SAM_EDGE);
    let mut out = vec![0u8; SAM_EDGE * SAM_EDGE * 3];
    for y in 0..nh {
        let sy = (((y as f32 + 0.5) / scale - 0.5).round() as usize).min(height as usize - 1);
        for x in 0..nw {
            let sx = (((x as f32 + 0.5) / scale - 0.5).round() as usize).min(width as usize - 1);
            let src = (sy * width as usize + sx) * 3;
            let dst = (y * SAM_EDGE + x) * 3;
            out[dst] = rgb8[src];
            out[dst + 1] = rgb8[src + 1];
            out[dst + 2] = rgb8[src + 2];
        }
    }
    (out, scale)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letterbox_scales_long_edge_and_pads() {
        let width = 40u32;
        let height = 20u32;
        let rgb = vec![200u8; (width * height * 3) as usize];
        let (out, scale) = letterbox(&rgb, width, height);
        assert_eq!(out.len(), SAM_EDGE * SAM_EDGE * 3);
        assert!((scale - SAM_EDGE as f32 / 40.0).abs() < 1e-6);
        assert_eq!(out[0], 200);
        let bottom = (SAM_EDGE - 1) * SAM_EDGE * 3;
        assert_eq!(out[bottom], 0);
    }

    #[test]
    fn sigmoid_maps_logits_into_unit_range() {
        assert!(sigmoid(0.0) - 0.5 < 1e-6);
        assert!(sigmoid(10.0) > 0.99);
        assert!(sigmoid(-10.0) < 0.01);
    }
}
