pub mod cache;
pub mod cpu;
pub mod decode;
pub mod edits;
pub mod encode;
pub mod frame;
pub mod gpu;
pub mod histogram;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("decode: {0}")]
    Decode(String),
    #[error("encode: {0}")]
    Encode(String),
    #[error("render: {0}")]
    Render(String),
    #[error("unsupported: {0}")]
    Unsupported(String),
}

pub type PipelineResult<T> = Result<T, PipelineError>;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub use cpu::CpuRenderer;
pub use frame::{RawFrame, RenderOptions, RenderedImage, Renderer};
pub use gpu::GpuRenderer;
pub use gpu::context::GpuContext;
