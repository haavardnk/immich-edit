pub mod client;
pub mod dto;
mod original;
pub mod url;

pub use client::ImmichClient;

#[derive(Debug, Clone, thiserror::Error)]
pub enum ImmichError {
    #[error("upstream unauthorized")]
    Unauthorized,
    #[error("upstream not found")]
    NotFound,
    #[error("upstream timeout")]
    Timeout,
    #[error("upstream transport error: {0}")]
    Transport(String),
    #[error("upstream rejected ({status}): {message}")]
    Rejected { status: u16, message: String },
    #[error("upstream redirected ({0})")]
    Redirect(u16),
    #[error("upstream status {0}")]
    Status(u16),
    #[error("upstream decode error: {0}")]
    Decode(String),
}

pub type ImmichResult<T> = Result<T, ImmichError>;

impl ImmichError {
    pub fn short(&self) -> String {
        match self {
            Self::Unauthorized => "unauthorized".into(),
            Self::NotFound => "not found".into(),
            Self::Timeout => "timeout".into(),
            Self::Rejected { message, .. } => message.clone(),
            Self::Redirect(code) => format!("redirected (HTTP {code})"),
            Self::Status(code) => format!("status {code}"),
            Self::Transport(_) => "transport error".into(),
            Self::Decode(_) => "decode error".into(),
        }
    }
}

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
                "Immich rejected the current session".to_string(),
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
            ImmichError::Redirect(code) => (
                "redirected",
                format!(
                    "Immich redirected with HTTP {code}; an access proxy may be in front of it"
                ),
                Some(code),
            ),
            ImmichError::Status(code @ (502 | 503)) => (
                "upstream_5xx",
                format!("Immich returned {code} after retries"),
                Some(code),
            ),
            ImmichError::Status(code) | ImmichError::Rejected { status: code, .. } => (
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
