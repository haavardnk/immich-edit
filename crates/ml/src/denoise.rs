use std::path::Path;

use ort::value::TensorRef;

use crate::model::ModelInput;
use crate::runtime::{
    Backend, ModelError, ModelRuntime, RuntimeMode, SessionConfig, extract_f32, ort_err,
};

pub struct Denoiser {
    runtime: ModelRuntime,
    input: String,
    output: String,
    edge: usize,
}

impl Denoiser {
    pub fn open(
        path: &Path,
        mode: RuntimeMode,
        config: &SessionConfig,
    ) -> Result<Self, ModelError> {
        let runtime = ModelRuntime::open(path, mode, config)?;
        let declared = runtime.first_input()?;
        let input = declared.name().to_string();
        let edge = declared
            .dtype()
            .tensor_shape()
            .and_then(|shape| shape.last().copied())
            .filter(|edge| *edge > 0)
            .map_or(ModelInput::NIND_DENOISE.edge() as usize, |edge| {
                edge as usize
            });
        let output = runtime.first_output()?.name().to_string();
        Ok(Self {
            runtime,
            input,
            output,
            edge,
        })
    }

    pub fn backend(&self) -> Backend {
        self.runtime.backend
    }

    pub fn tile_edge(&self) -> usize {
        self.edge
    }

    pub fn run(&mut self, chw: &[f32]) -> Result<Vec<f32>, ModelError> {
        let shape = vec![1usize, 3, self.edge, self.edge];
        let outputs = self
            .runtime
            .session
            .run(ort::inputs![
                self.input.as_str() => TensorRef::from_array_view((shape, chw)).map_err(ort_err)?
            ])
            .map_err(ort_err)?;
        let (_, values) = extract_f32(&outputs[self.output.as_str()])?;
        Ok(values)
    }
}
