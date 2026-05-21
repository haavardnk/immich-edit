pub mod client;
pub mod dto;

pub use client::ImmichClient;

#[derive(Debug, thiserror::Error)]
pub enum ImmichError {
    #[error("upstream unauthorized")]
    Unauthorized,
    #[error("upstream not found")]
    NotFound,
    #[error("upstream timeout")]
    Timeout,
    #[error("upstream transport error")]
    Transport(String),
    #[error("upstream status {0}")]
    Status(u16),
    #[error("upstream decode error: {0}")]
    Decode(String),
}

pub type ImmichResult<T> = Result<T, ImmichError>;
