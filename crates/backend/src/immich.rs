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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ImmichConnectionStatus {
    pub ok: bool,
    pub kind: &'static str,
    pub message: String,
    pub status_code: Option<u16>,
}

impl ImmichConnectionStatus {
    pub fn from_ping(result: ImmichResult<()>) -> Self {
        let Err(err) = result else {
            return Self {
                ok: true,
                kind: "ok",
                message: "Immich is reachable".into(),
                status_code: None,
            };
        };
        let (kind, message, status_code) = match err {
            ImmichError::Unauthorized => (
                "api_key_rejected",
                "Immich rejected IMMICH_API_KEY".to_string(),
                None,
            ),
            ImmichError::Timeout => (
                "timeout",
                "Immich did not respond before the configured timeout".into(),
                None,
            ),
            ImmichError::Transport(_) => (
                "unreachable",
                "Immich is unreachable from immich-edit".into(),
                None,
            ),
            ImmichError::Status(code @ (502 | 503)) => (
                "upstream_5xx",
                format!("Immich returned {code} after retries"),
                Some(code),
            ),
            ImmichError::Status(code) => (
                "http_status",
                format!("Immich returned HTTP {code}"),
                Some(code),
            ),
            ImmichError::NotFound => (
                "not_found",
                "Immich ping endpoint was not found".into(),
                Some(404),
            ),
            ImmichError::Decode(_) => (
                "invalid_response",
                "Immich returned an unexpected response".into(),
                None,
            ),
        };
        Self {
            ok: false,
            kind,
            message,
            status_code,
        }
    }
}
