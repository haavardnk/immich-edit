pub mod auto;
pub mod cancel;
pub mod cpu;
pub mod decode;
pub mod edit_manifest;
pub mod edits;
pub mod encode;
pub mod exif;
pub mod frame;
pub mod gpu;
pub mod histogram;
pub mod ops;

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
    #[error("cancelled")]
    Cancelled,
}

pub type PipelineResult<T> = Result<T, PipelineError>;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub use cancel::{CancelToken, CancelTracker};
pub use frame::{RawFrame, RenderOptions, RenderedImage};
pub use gpu::GpuRenderer;
pub use gpu::context::GpuContext;
