use std::path::Path;
use std::sync::LazyLock;

use ort::environment::Environment;
use ort::memory::{AllocationDevice, AllocatorType, MemoryInfo, MemoryType};
use ort::session::Session;
use ort::session::builder::SessionBuilder;
use ort::value::{DynValue, Outlet};

mod ort_log;
#[cfg(test)]
mod test_logs;

#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("onnxruntime: {0}")]
    Ort(String),
    #[error("invalid input: {0}")]
    Input(String),
    #[error("unusable model: {0}")]
    Model(String),
}

pub fn ort_err<R>(e: ort::Error<R>) -> ModelError {
    ModelError::Ort(e.to_string())
}

pub(crate) fn extract_f32(value: &DynValue) -> Result<(Vec<i64>, Vec<f32>), ModelError> {
    if let Ok((dims, v)) = value.try_extract_tensor::<f32>() {
        return Ok((dims.to_vec(), v.to_vec()));
    }
    let (dims, v) = value.try_extract_tensor::<half::f16>().map_err(ort_err)?;
    Ok((dims.to_vec(), v.iter().map(|h| h.to_f32()).collect()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RuntimeMode {
    #[default]
    Auto,
    Gpu,
    Cpu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    WebGpu,
    Cpu,
}

impl Backend {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WebGpu => "webgpu",
            Self::Cpu => "cpu",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SessionConfig {
    pub intra_threads: usize,
    pub memory_pattern: bool,
    pub arena: bool,
    pub force_cpu_nodes: Vec<String>,
}

pub struct ModelRuntime {
    pub session: Session,
    pub backend: Backend,
}

impl ModelRuntime {
    pub fn open(
        path: &Path,
        mode: RuntimeMode,
        config: &SessionConfig,
    ) -> Result<Self, ModelError> {
        if mode != RuntimeMode::Cpu {
            match build_webgpu(path, config) {
                Ok(session) => {
                    return Ok(Self {
                        session,
                        backend: Backend::WebGpu,
                    });
                }
                Err(e) => {
                    if mode == RuntimeMode::Gpu {
                        return Err(e);
                    }
                    tracing::warn!("webgpu execution provider unavailable, using cpu: {e}");
                }
            }
        }
        let session = base_builder(config)?
            .commit_from_file(path)
            .map_err(ort_err)?;
        Ok(Self {
            session,
            backend: Backend::Cpu,
        })
    }

    pub fn first_input(&self) -> Result<&Outlet, ModelError> {
        self.session
            .inputs()
            .first()
            .ok_or_else(|| ModelError::Model("declares no inputs".into()))
    }

    pub fn first_output(&self) -> Result<&Outlet, ModelError> {
        self.session
            .outputs()
            .first()
            .ok_or_else(|| ModelError::Model("declares no outputs".into()))
    }
}

static ENVIRONMENT: LazyLock<Result<Environment, String>> = LazyLock::new(|| {
    let environment = ort_log::routed(ort::init())
        .build()
        .map_err(|e| e.to_string())?;
    environment.set_log_level(ort_log::threshold());
    Ok(environment)
});

fn base_builder(config: &SessionConfig) -> Result<SessionBuilder, ModelError> {
    let environment = ENVIRONMENT
        .as_ref()
        .map_err(|e| ModelError::Ort(e.clone()))?;
    let mut builder = Session::builder(environment)
        .map_err(ort_err)?
        .with_memory_pattern(config.memory_pattern)
        .map_err(ort_err)?;
    if config.intra_threads > 0 {
        builder = builder
            .with_intra_threads(config.intra_threads)
            .map_err(ort_err)?;
    }
    if !config.arena {
        let info = MemoryInfo::new(
            AllocationDevice::CPU,
            0,
            AllocatorType::Device,
            MemoryType::Default,
        )
        .map_err(ort_err)?;
        builder = builder.with_allocator(info).map_err(ort_err)?;
    }
    Ok(builder)
}

#[cfg(not(all(target_arch = "aarch64", target_os = "linux")))]
fn build_webgpu(path: &Path, config: &SessionConfig) -> Result<Session, ModelError> {
    use ort::ep::WebGPU;

    let mut ep = WebGPU::default();
    if !config.force_cpu_nodes.is_empty() {
        ep = ep.with_force_cpu_node_names(config.force_cpu_nodes.join("\n"));
    }
    base_builder(config)?
        .with_execution_providers([ep.build().error_on_failure()])
        .map_err(ort_err)?
        .commit_from_file(path)
        .map_err(ort_err)
}

#[cfg(all(target_arch = "aarch64", target_os = "linux"))]
fn build_webgpu(_path: &Path, _config: &SessionConfig) -> Result<Session, ModelError> {
    Err(ModelError::Ort(
        "onnxruntime ships no webgpu build for aarch64 linux".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn varint(mut v: u64) -> Vec<u8> {
        let mut out = Vec::new();
        while v >= 0x80 {
            out.push((v as u8) | 0x80);
            v >>= 7;
        }
        out.push(v as u8);
        out
    }

    fn int_field(field: u64, v: u64) -> Vec<u8> {
        [varint(field << 3), varint(v)].concat()
    }

    fn bytes_field(field: u64, bytes: &[u8]) -> Vec<u8> {
        [
            varint((field << 3) | 2),
            varint(bytes.len() as u64),
            bytes.to_vec(),
        ]
        .concat()
    }

    fn constant_only_model() -> Vec<u8> {
        let tensor = [
            int_field(1, 1),
            int_field(2, 1),
            bytes_field(4, &0f32.to_le_bytes()),
        ]
        .concat();
        let attribute = [
            bytes_field(1, b"value"),
            bytes_field(5, &tensor),
            int_field(20, 4),
        ]
        .concat();
        let node = [
            bytes_field(2, b"y"),
            bytes_field(4, b"Constant"),
            bytes_field(5, &attribute),
        ]
        .concat();
        let shape = bytes_field(1, &int_field(1, 1));
        let tensor_type = [int_field(1, 1), bytes_field(2, &shape)].concat();
        let value_info = [
            bytes_field(1, b"y"),
            bytes_field(2, &bytes_field(1, &tensor_type)),
        ]
        .concat();
        let graph = [
            bytes_field(1, &node),
            bytes_field(2, b"g"),
            bytes_field(12, &value_info),
        ]
        .concat();
        [
            int_field(1, 7),
            bytes_field(7, &graph),
            bytes_field(8, &int_field(2, 13)),
        ]
        .concat()
    }

    #[test]
    fn opening_a_session_logs_nothing_at_info() {
        let path = std::env::temp_dir().join(format!("ml-quiet-{}.onnx", std::process::id()));
        std::fs::write(&path, constant_only_model()).unwrap();
        let (logs, guard) = test_logs::capture(tracing::Level::INFO);
        let runtime = ModelRuntime::open(&path, RuntimeMode::Cpu, &SessionConfig::default());
        drop(guard);
        std::fs::remove_file(&path).unwrap();
        runtime.unwrap();

        let text = logs.text();
        assert!(!text.contains(" INFO "), "{text}");
    }

    #[test]
    fn a_model_without_inputs_is_an_error_not_a_panic() {
        let path = std::env::temp_dir().join(format!("ml-no-inputs-{}.onnx", std::process::id()));
        std::fs::write(&path, constant_only_model()).unwrap();
        let runtime = ModelRuntime::open(&path, RuntimeMode::Cpu, &SessionConfig::default());
        std::fs::remove_file(&path).unwrap();
        let runtime = runtime.unwrap();

        assert!(matches!(runtime.first_input(), Err(ModelError::Model(_))));
        assert_eq!(runtime.first_output().unwrap().name(), "y");
    }
}
