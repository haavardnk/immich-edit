use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("internal: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            AppError::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL", msg.clone())
            }
        };

        let body = json!({
            "code": code,
            "message": message,
        });

        (status, Json(body)).into_response()
    }
}
