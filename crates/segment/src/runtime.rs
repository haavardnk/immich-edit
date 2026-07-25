use std::path::Path;

use ort::memory::{AllocationDevice, AllocatorType, MemoryInfo, MemoryType};
use ort::session::Session;
use ort::session::builder::SessionBuilder;

#[derive(Debug, thiserror::Error)]
pub enum SegmentError {
    #[error("onnxruntime: {0}")]
    Ort(String),
    #[error("invalid input: {0}")]
    Input(String),
}

pub fn ort_err<R>(e: ort::Error<R>) -> SegmentError {
    SegmentError::Ort(e.to_string())
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

pub struct SegmentRuntime {
    pub session: Session,
    pub backend: Backend,
}

impl SegmentRuntime {
    pub fn open(
        path: &Path,
        mode: RuntimeMode,
        config: &SessionConfig,
    ) -> Result<Self, SegmentError> {
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
}

fn base_builder(config: &SessionConfig) -> Result<SessionBuilder, SegmentError> {
    let mut builder = Session::builder()
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

#[cfg(feature = "webgpu")]
fn build_webgpu(path: &Path, config: &SessionConfig) -> Result<Session, SegmentError> {
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

#[cfg(not(feature = "webgpu"))]
fn build_webgpu(_path: &Path, _config: &SessionConfig) -> Result<Session, SegmentError> {
    Err(SegmentError::Ort("built without the webgpu feature".into()))
}
