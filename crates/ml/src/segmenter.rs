use std::path::Path;

use ort::value::{DynValue, TensorRef};

use crate::image::{Placement, activate, mask_to_source, to_tensor};
use crate::model::{Activation, Layout, ModelSpec};
use crate::runtime::{Backend, RuntimeMode, SegmentError, SegmentRuntime, SessionConfig};

pub struct Mask {
    pub values: Vec<f32>,
    pub width: usize,
    pub height: usize,
}

pub struct Segmenter {
    runtime: SegmentRuntime,
    spec: ModelSpec,
}

impl Segmenter {
    pub fn open(
        path: &Path,
        spec: ModelSpec,
        mode: RuntimeMode,
        config: &SessionConfig,
    ) -> Result<Self, SegmentError> {
        let runtime = SegmentRuntime::open(path, mode, config)?;
        Ok(Self { runtime, spec })
    }

    pub fn backend(&self) -> Backend {
        self.runtime.backend
    }

    pub fn spec(&self) -> &ModelSpec {
        &self.spec
    }

    pub fn run(&mut self, rgb8: &[u8], width: u32, height: u32) -> Result<Mask, SegmentError> {
        self.run_with(rgb8, width, height, self.spec.activation)
    }

    pub fn run_with(
        &mut self,
        rgb8: &[u8],
        width: u32,
        height: u32,
        activation: Activation,
    ) -> Result<Mask, SegmentError> {
        if rgb8.len() != (width as usize) * (height as usize) * 3 {
            return Err(SegmentError::Input(
                "rgb8 length does not match dimensions".into(),
            ));
        }
        let (tensor, place) = to_tensor(rgb8, width, height, &self.spec);
        let edge = self.spec.input_edge as usize;
        let shape = match self.spec.layout {
            Layout::Nchw => vec![1usize, 3, edge, edge],
            Layout::Nhwc => vec![1usize, edge, edge, 3],
        };

        let input_name = self.runtime.session.inputs()[0].name().to_string();
        let output_name = self.runtime.session.outputs()[0].name().to_string();
        let declared: Vec<i64> = self.runtime.session.outputs()[0]
            .dtype()
            .tensor_shape()
            .map(|s| s.to_vec())
            .unwrap_or_default();

        let outputs = self
            .runtime
            .session
            .run(ort::inputs![
                input_name.as_str() => TensorRef::from_array_view((shape, tensor.as_slice()))
                    .map_err(crate::runtime::ort_err)?
            ])
            .map_err(crate::runtime::ort_err)?;

        let (runtime_dims, values) = extract_f32(&outputs[output_name.as_str()])?;
        let out_dims = if runtime_dims.iter().any(|d| *d > 0) {
            runtime_dims
        } else {
            declared
        };
        let (mw, mh) = plane_dims(&out_dims, values.len());
        let plane = mw * mh;
        if values.len() < plane {
            return Err(SegmentError::Input(format!(
                "output has {} values, expected at least {plane}",
                values.len()
            )));
        }
        let activated = activate(&values, mw, mh, activation);
        let full = mask_to_source(
            &activated[..plane.min(activated.len())],
            mw,
            mh,
            &place,
            width as usize,
            height as usize,
        );
        Ok(Mask {
            values: full,
            width: width as usize,
            height: height as usize,
        })
    }
}

fn plane_dims(dims: &[i64], len: usize) -> (usize, usize) {
    if dims.len() >= 2 {
        let h = dims[dims.len() - 2];
        let w = dims[dims.len() - 1];
        if h > 0 && w > 0 {
            return (w as usize, h as usize);
        }
    }
    if dims.len() >= 3 {
        let classes = dims[dims.len() - 3];
        if classes > 0 && len % (classes as usize) == 0 {
            let plane = len / (classes as usize);
            let edge = (plane as f64).sqrt() as usize;
            if edge * edge == plane {
                return (edge, edge);
            }
        }
    }
    let edge = (len as f64).sqrt() as usize;
    (edge, edge)
}

fn extract_f32(value: &DynValue) -> Result<(Vec<i64>, Vec<f32>), SegmentError> {
    if let Ok((dims, v)) = value.try_extract_tensor::<f32>() {
        return Ok((dims.to_vec(), v.to_vec()));
    }
    let (dims, v) = value
        .try_extract_tensor::<half::f16>()
        .map_err(crate::runtime::ort_err)?;
    Ok((dims.to_vec(), v.iter().map(|h| h.to_f32()).collect()))
}

pub fn placement_for(spec: &ModelSpec, width: u32, height: u32) -> Placement {
    Placement::compute(spec.fit, spec.input_edge as usize, width, height)
}

#[cfg(test)]
mod tests {
    use super::plane_dims;

    #[test]
    fn static_dims_win() {
        assert_eq!(plane_dims(&[1, 1, 96, 128], 12288), (128, 96));
    }

    #[test]
    fn dynamic_edges_fall_back_to_the_class_count() {
        let len = 150 * 128 * 128;
        assert_eq!(plane_dims(&[1, 150, -1, -1], len), (128, 128));
        assert_eq!(plane_dims(&[1, 150, 0, 0], len), (128, 128));
    }

    #[test]
    fn single_plane_still_uses_the_square_root() {
        assert_eq!(plane_dims(&[1, 1, -1, -1], 64), (8, 8));
        assert_eq!(plane_dims(&[], 64), (8, 8));
    }
}
