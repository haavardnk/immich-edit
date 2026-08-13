use std::path::Path;

use ort::session::SessionInputValue;
use ort::value::TensorRef;

use crate::image::{Placement, activate, mask_to_source, to_tensor};
use crate::model::{Activation, ModelSpec};
use crate::runtime::{Backend, RuntimeMode, SegmentError, SegmentRuntime, SessionConfig, ort_err};
use crate::segmenter::Mask;

pub const SAM_EDGE: usize = 1024;
const LOW_RES: usize = 256;
const BOX_TOP_LEFT: f32 = 2.0;
const BOX_BOTTOM_RIGHT: f32 = 3.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClickPoint {
    pub x: f32,
    pub y: f32,
    pub positive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoxPrompt {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EmbeddingTensor {
    pub name: String,
    pub dims: Vec<i64>,
    pub values: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct Embedding {
    pub tensors: Vec<EmbeddingTensor>,
    pub scale: f32,
    pub width: u32,
    pub height: u32,
}

const PROMPT_INPUTS: [&str; 5] = [
    "point_coords",
    "point_labels",
    "mask_input",
    "has_mask_input",
    "orig_im_size",
];

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
        let (chw, _) = to_tensor(rgb8, width, height, &ModelSpec::SAM);
        let scale = Placement::contain_scale(SAM_EDGE, width, height);

        let input = self.runtime.session.inputs()[0].name().to_string();
        let names: Vec<String> = self
            .runtime
            .session
            .outputs()
            .iter()
            .map(|o| o.name().to_string())
            .collect();
        let outputs = self
            .runtime
            .session
            .run(ort::inputs![
                input.as_str() => TensorRef::from_array_view(
                    (vec![1usize, 3, SAM_EDGE, SAM_EDGE], chw.as_slice()),
                ).map_err(ort_err)?
            ])
            .map_err(ort_err)?;

        let mut tensors = Vec::with_capacity(names.len());
        for name in names {
            let (dims, values) = outputs[name.as_str()]
                .try_extract_tensor::<f32>()
                .map_err(ort_err)?;
            tensors.push(EmbeddingTensor {
                name,
                dims: dims.to_vec(),
                values: values.to_vec(),
            });
        }
        Ok(Embedding {
            tensors,
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
        self.decode_with(embedding, points, None)
    }

    pub fn decode_with(
        &mut self,
        embedding: &Embedding,
        points: &[ClickPoint],
        bbox: Option<BoxPrompt>,
    ) -> Result<Mask, SegmentError> {
        if points.is_empty() && bbox.is_none() {
            return Err(SegmentError::Input("no points given".into()));
        }
        let prompts = points.len() + if bbox.is_some() { 2 } else { 0 };
        let mut coords: Vec<f32> = Vec::with_capacity(prompts * 2);
        let mut labels: Vec<f32> = Vec::with_capacity(prompts);
        for p in points {
            coords.push(p.x * embedding.scale);
            coords.push(p.y * embedding.scale);
            labels.push(if p.positive { 1.0 } else { 0.0 });
        }
        if let Some(b) = bbox {
            coords.push(b.x0.min(b.x1) * embedding.scale);
            coords.push(b.y0.min(b.y1) * embedding.scale);
            labels.push(BOX_TOP_LEFT);
            coords.push(b.x0.max(b.x1) * embedding.scale);
            coords.push(b.y0.max(b.y1) * embedding.scale);
            labels.push(BOX_BOTTOM_RIGHT);
        }
        let mask_input = vec![0f32; LOW_RES * LOW_RES];
        let has_mask = vec![0f32; 1];
        let orig_size = vec![embedding.height as f32, embedding.width as f32];

        let names: Vec<String> = self
            .runtime
            .session
            .inputs()
            .iter()
            .map(|i| i.name().to_string())
            .collect();
        let slots = embedding_slots(&names, &embedding.tensors)?;
        let mask_name = self.runtime.session.outputs()[0].name().to_string();

        let mut feeds: Vec<(&str, SessionInputValue)> = Vec::with_capacity(names.len());
        for name in &names {
            let value = match name.as_str() {
                "point_coords" => {
                    TensorRef::from_array_view((vec![1usize, prompts, 2], coords.as_slice()))
                }
                "point_labels" => {
                    TensorRef::from_array_view((vec![1usize, prompts], labels.as_slice()))
                }
                "mask_input" => TensorRef::from_array_view((
                    vec![1usize, 1, LOW_RES, LOW_RES],
                    mask_input.as_slice(),
                )),
                "has_mask_input" => TensorRef::from_array_view((vec![1usize], has_mask.as_slice())),
                "orig_im_size" => TensorRef::from_array_view((vec![2usize], orig_size.as_slice())),
                _ => {
                    let tensor = &embedding.tensors[slots[name]];
                    let shape: Vec<usize> = tensor.dims.iter().map(|d| *d as usize).collect();
                    TensorRef::from_array_view((shape, tensor.values.as_slice()))
                }
            };
            feeds.push((name.as_str(), value.map_err(ort_err)?.into()));
        }

        let outputs = self.runtime.session.run(feeds).map_err(ort_err)?;

        let (dims, values) = outputs[mask_name.as_str()]
            .try_extract_tensor::<f32>()
            .map_err(ort_err)?;
        if dims.len() < 2 {
            return Err(SegmentError::Input("decoder output has no plane".into()));
        }
        let h = dims[dims.len() - 2] as usize;
        let w = dims[dims.len() - 1] as usize;
        let plane = w * h;
        if plane == 0 || values.len() < plane {
            return Err(SegmentError::Input(format!(
                "decoder returned {} values, expected at least {plane}",
                values.len()
            )));
        }
        let candidates = values.len() / plane;
        let best = outputs
            .get("iou_predictions")
            .and_then(|v| v.try_extract_tensor::<f32>().ok())
            .map(|(_, iou)| best_candidate(iou, candidates))
            .unwrap_or(0);
        let logits = &values[best * plane..best * plane + plane];
        let probs = activate(logits, w, h, Activation::Sigmoid);
        let out_w = embedding.width as usize;
        let out_h = embedding.height as usize;

        if w == out_w && h == out_h {
            return Ok(Mask {
                values: probs,
                width: w,
                height: h,
            });
        }
        let place = Placement::compute(
            ModelSpec::SAM.fit,
            SAM_EDGE,
            embedding.width,
            embedding.height,
        );
        Ok(Mask {
            values: mask_to_source(&probs, w, h, &place, out_w, out_h),
            width: out_w,
            height: out_h,
        })
    }
}

fn embedding_slots(
    inputs: &[String],
    tensors: &[EmbeddingTensor],
) -> Result<std::collections::HashMap<String, usize>, SegmentError> {
    let mut slots = std::collections::HashMap::new();
    let mut taken = vec![false; tensors.len()];
    let wanted: Vec<&String> = inputs
        .iter()
        .filter(|n| !PROMPT_INPUTS.contains(&n.as_str()))
        .collect();
    for name in &wanted {
        if let Some(i) = tensors.iter().position(|t| &&t.name == name) {
            taken[i] = true;
            slots.insert((*name).clone(), i);
        }
    }
    for name in &wanted {
        if slots.contains_key(*name) {
            continue;
        }
        let Some(i) = taken.iter().position(|t| !t) else {
            return Err(SegmentError::Input(format!(
                "decoder input {name} has no matching encoder output"
            )));
        };
        taken[i] = true;
        slots.insert((*name).clone(), i);
    }
    Ok(slots)
}

fn best_candidate(iou: &[f32], candidates: usize) -> usize {
    if candidates < 2 || iou.len() < candidates {
        return 0;
    }
    iou[..candidates]
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .map(|(i, _)| i)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sam_pads_bottom_right_so_prompts_need_no_offset() {
        let place = Placement::compute(ModelSpec::SAM.fit, SAM_EDGE, 40, 20);
        assert_eq!((place.x, place.y), (0, 0));
        assert_eq!((place.w, place.h), (SAM_EDGE, SAM_EDGE / 2));
        let scale = Placement::contain_scale(SAM_EDGE, 40, 20);
        assert!((scale - SAM_EDGE as f32 / 40.0).abs() < 1e-6);
    }

    fn tensor(name: &str) -> EmbeddingTensor {
        EmbeddingTensor {
            name: name.into(),
            dims: vec![1],
            values: vec![0.0],
        }
    }

    #[test]
    fn unnamed_embeddings_fall_back_to_encoder_order() {
        let inputs = vec![
            "image_embeddings".to_string(),
            "point_coords".to_string(),
            "orig_im_size".to_string(),
        ];
        let slots = embedding_slots(&inputs, &[tensor("embeddings")]).unwrap();
        assert_eq!(slots.len(), 1);
        assert_eq!(slots["image_embeddings"], 0);
    }

    #[test]
    fn matching_names_bind_across_encoder_output_order() {
        let inputs = vec![
            "image_embed".to_string(),
            "high_res_feats_0".to_string(),
            "high_res_feats_1".to_string(),
            "point_labels".to_string(),
        ];
        let tensors = [
            tensor("high_res_feats_0"),
            tensor("high_res_feats_1"),
            tensor("image_embed"),
        ];
        let slots = embedding_slots(&inputs, &tensors).unwrap();
        assert_eq!(slots["image_embed"], 2);
        assert_eq!(slots["high_res_feats_0"], 0);
        assert_eq!(slots["high_res_feats_1"], 1);
    }

    #[test]
    fn missing_embeddings_are_an_error() {
        let inputs = vec!["a".to_string(), "b".to_string()];
        assert!(embedding_slots(&inputs, &[tensor("a")]).is_err());
    }

    #[test]
    fn the_highest_scoring_candidate_wins() {
        assert_eq!(best_candidate(&[0.1, 0.9, 0.4], 3), 1);
        assert_eq!(best_candidate(&[0.7], 1), 0);
        assert_eq!(best_candidate(&[], 3), 0);
    }

    #[test]
    fn decoder_masks_crop_the_padding_and_rescale() {
        let place = Placement::compute(ModelSpec::SAM.fit, SAM_EDGE, 8, 4);
        let mut logits = vec![-10f32; 64];
        for y in 0..4 {
            for x in 0..8 {
                logits[y * 8 + x] = 10.0;
            }
        }
        let probs = activate(&logits, 8, 8, Activation::Sigmoid);
        let values = mask_to_source(&probs, 8, 8, &place, 8, 4);
        assert_eq!(values.len(), 32);
        assert!(values.iter().all(|v| *v > 0.99));
    }
}
