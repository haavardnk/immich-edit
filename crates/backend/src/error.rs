use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::immich::ImmichError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("upstream auth failed")]
    UpstreamAuth,
    #[error("upstream unavailable")]
    UpstreamUnavailable,
    #[error("upstream timeout")]
    UpstreamTimeout,
    #[error("internal error")]
    Internal,
    #[error("superseded")]
    Superseded,
}

impl AppError {
    fn parts(&self) -> (StatusCode, &'static str, String) {
        match self {
            Self::NotFound => (
                StatusCode::NOT_FOUND,
                "not_found",
                "resource not found".into(),
            ),
            Self::BadRequest(m) => (StatusCode::BAD_REQUEST, "bad_request", m.clone()),
            Self::UpstreamAuth => (
                StatusCode::BAD_GATEWAY,
                "upstream_auth",
                "upstream rejected credentials".into(),
            ),
            Self::UpstreamUnavailable => (
                StatusCode::BAD_GATEWAY,
                "upstream_unavailable",
                "upstream unavailable".into(),
            ),
            Self::UpstreamTimeout => (
                StatusCode::GATEWAY_TIMEOUT,
                "upstream_timeout",
                "upstream timed out".into(),
            ),
            Self::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal",
                "internal error".into(),
            ),
            Self::Superseded => (
                StatusCode::CONFLICT,
                "superseded",
                "superseded by newer render".into(),
            ),
        }
    }
}

impl From<ImmichError> for AppError {
    fn from(err: ImmichError) -> Self {
        match err {
            ImmichError::Unauthorized => Self::UpstreamAuth,
            ImmichError::NotFound => Self::NotFound,
            ImmichError::Timeout => Self::UpstreamTimeout,
            ImmichError::Transport(_) | ImmichError::Status(_) | ImmichError::Decode(_) => {
                Self::UpstreamUnavailable
            }
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let request_id = Uuid::new_v4();
        let (status, code, message) = self.parts();
        let body: Value = json!({
            "code": code,
            "message": message,
            "request_id": request_id,
        });
        if status.is_server_error() || status == StatusCode::BAD_GATEWAY {
            tracing::warn!(target: "app::error", %request_id, code, message, "request failed");
        }
        (status, Json(body)).into_response()
    }
}

pub async fn api_not_found() -> AppError {
    AppError::NotFound
}
