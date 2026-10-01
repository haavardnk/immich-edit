pub mod auto;
pub mod cancel;
pub mod capture_sigma;
pub mod color;
pub mod cpu;
pub mod dcp;
#[cfg(feature = "native")]
pub mod decode;
pub mod edit_manifest;
pub mod edits;
#[cfg(feature = "native")]
pub mod encode;
#[cfg(feature = "native")]
pub mod exif;
#[cfg(feature = "native")]
pub mod finish;
pub mod frame;
pub mod geom;
pub mod gpu;
pub mod histogram;
pub mod lut;
pub mod mask_raster;
pub mod math;
pub mod ops;
pub mod scopes;
mod sensor_sample;
pub mod source;
pub mod timing;
pub mod tone;
pub mod warn;
pub mod white_balance;

use thiserror::Error;

#[derive(Debug, Clone, Error)]
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
    #[error("gpu device lost")]
    DeviceLost,
}

pub type PipelineResult<T> = Result<T, PipelineError>;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub use cancel::{CancelToken, CancelTracker};
#[cfg(feature = "native")]
pub use cpu::CpuRenderer;
pub use dcp::{DcpProfile, parse_dcp};
pub use frame::{
    BitDepth, FrameMeta, OutputFormat, PngCompression, RawFrame, RenderOptions, RenderedImage,
    TiffCompression,
};
pub use gpu::GpuPoolStats;
pub use gpu::GpuRenderer;
pub use gpu::GpuRendererOptions;
pub use gpu::context::GpuContext;
pub use lut::{CubeLut, LutMap, empty_luts};
pub use mask_raster::{MaskRaster, RasterMap, empty_rasters};
