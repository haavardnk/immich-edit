use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use raw_pipeline::PipelineError;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::immich::ImmichError;
use crate::services::edited_thumb::EditedThumbError;
use crate::services::render::RenderError;
use crate::telemetry::ErrorChain;
use crate::telemetry::http::ErrorReport;

tokio::task_local! {
    pub static REQUEST_ID: String;
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("upstream auth failed")]
    UpstreamAuth,
    #[error("upstream unavailable")]
    UpstreamUnavailable,
    #[error("upstream rejected: {0}")]
    UpstreamRejected(String),
    #[error("upstream timeout")]
    UpstreamTimeout,
    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),
    #[error("unprocessable: {0}")]
    Unprocessable(String),
    #[error("internal error")]
    Internal,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("superseded")]
    Superseded,
    #[error("setup required")]
    SetupRequired,
    #[error("admin required")]
    AdminRequired,
    #[error("forbidden")]
    Forbidden,
    #[error("access disabled")]
    AccessDisabled,
    #[error("rate limited")]
    RateLimited(Option<u64>),
}

impl AppError {
    pub fn internal(context: &'static str, err: &(dyn std::error::Error + 'static)) -> Self {
        tracing::error!(error = %ErrorChain(err), "{context}");
        Self::Internal
    }

    fn parts(&self) -> (StatusCode, &'static str, String) {
        match self {
            Self::NotFound => (
                StatusCode::NOT_FOUND,
                "not_found",
                "resource not found".into(),
            ),
            Self::BadRequest(m) => (StatusCode::BAD_REQUEST, "bad_request", m.clone()),
            Self::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "authentication required".into(),
            ),
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
            Self::UpstreamRejected(m) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "upstream_rejected",
                format!("Immich refused the request: {m}"),
            ),
            Self::UpstreamTimeout => (
                StatusCode::GATEWAY_TIMEOUT,
                "upstream_timeout",
                "upstream timed out".into(),
            ),
            Self::UnsupportedFormat(m) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "unsupported_format",
                m.clone(),
            ),
            Self::Unprocessable(m) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "unprocessable", m.clone())
            }
            Self::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal",
                "internal error".into(),
            ),
            Self::Conflict(m) => (StatusCode::CONFLICT, "conflict", m.clone()),
            Self::Superseded => (
                StatusCode::CONFLICT,
                "superseded",
                "superseded by newer render".into(),
            ),
            Self::SetupRequired => (
                StatusCode::PRECONDITION_REQUIRED,
                "setup_required",
                "instance setup required".into(),
            ),
            Self::AdminRequired => (
                StatusCode::FORBIDDEN,
                "admin_required",
                "administrator access required".into(),
            ),
            Self::Forbidden => (
                StatusCode::FORBIDDEN,
                "forbidden",
                "request rejected".into(),
            ),
            Self::AccessDisabled => (
                StatusCode::FORBIDDEN,
                "access_disabled",
                "local access is disabled for this account".into(),
            ),
            Self::RateLimited(_) => (
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limited",
                "too many attempts; try again later".into(),
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
            ImmichError::Rejected { message, .. } => Self::UpstreamRejected(message),
            ImmichError::Status(_) | ImmichError::Transport(_) => Self::UpstreamUnavailable,
            ImmichError::Decode(detail) => {
                tracing::warn!(target: "upstream", detail = detail.as_str(), "immich response could not be decoded");
                Self::UpstreamUnavailable
            }
        }
    }
}

impl From<RenderError> for AppError {
    fn from(err: RenderError) -> Self {
        match err {
            RenderError::Upstream(e) => e.into(),
            RenderError::Pipeline(PipelineError::Unsupported(msg)) => Self::UnsupportedFormat(msg),
            RenderError::Pipeline(PipelineError::Cancelled) => Self::Superseded,
            RenderError::Pipeline(e) => Self::internal("render pipeline", &e),
            RenderError::Lut(m) | RenderError::Dcp(m) => Self::BadRequest(m),
        }
    }
}

impl From<EditedThumbError> for AppError {
    fn from(err: EditedThumbError) -> Self {
        match err {
            EditedThumbError::NotFound | EditedThumbError::HashMismatch => Self::NotFound,
            EditedThumbError::Render(e) => e.into(),
            EditedThumbError::Io(e) => Self::internal("edited thumb io", &e),
        }
    }
}

macro_rules! internal_from {
    ($ty:path, $ctx:literal) => {
        impl From<$ty> for AppError {
            fn from(err: $ty) -> Self {
                Self::internal($ctx, &err)
            }
        }
    };
}

macro_rules! store_from {
    ($ty:path, $ctx:literal) => {
        impl From<$ty> for AppError {
            fn from(err: $ty) -> Self {
                use $ty as E;
                match err {
                    E::NotFound => Self::NotFound,
                    E::Invalid(m) => Self::BadRequest(m),
                    e => Self::internal($ctx, &e),
                }
            }
        }
    };
    ($ty:path, $ctx:literal, dup) => {
        impl From<$ty> for AppError {
            fn from(err: $ty) -> Self {
                use $ty as E;
                match err {
                    E::NotFound => Self::NotFound,
                    E::Invalid(m) => Self::BadRequest(m),
                    E::Duplicate(meta) => {
                        Self::Conflict(format!(concat!($ctx, " already exists: {}"), meta.id))
                    }
                    e => Self::internal($ctx, &e),
                }
            }
        }
    };
}

internal_from!(crate::services::edits_store::EditsStoreError, "edits store");
internal_from!(crate::services::job_store::JobStoreError, "job store");
internal_from!(std::io::Error, "io");
store_from!(crate::services::dcp_store::DcpStoreError, "dcp", dup);
store_from!(crate::services::lut_store::LutStoreError, "lut", dup);
store_from!(
    crate::services::watermark_store::WatermarkStoreError,
    "watermark",
    dup
);
#[cfg(feature = "ml")]
store_from!(crate::services::model_store::ModelStoreError, "model store");
store_from!(
    crate::services::raster_store::RasterStoreError,
    "raster store"
);

macro_rules! configurable_from {
    ($ty:path, $ctx:literal) => {
        impl From<$ty> for AppError {
            fn from(err: $ty) -> Self {
                use $ty as E;
                match err {
                    E::AlreadyConfigured => Self::Conflict("instance already configured".into()),
                    e => Self::internal($ctx, &e),
                }
            }
        }
    };
}

configurable_from!(crate::services::auth_store::AuthStoreError, "auth store");
configurable_from!(
    crate::services::instance_store::InstanceStoreError,
    "instance store"
);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let request_id = REQUEST_ID
            .try_with(|s| s.clone())
            .unwrap_or_else(|_| Uuid::new_v4().to_string());
        let (status, code, message) = self.parts();
        let body: Value = json!({
            "code": code,
            "message": message,
            "request_id": request_id,
        });
        let mut resp = (status, Json(body)).into_response();
        resp.extensions_mut().insert(ErrorReport {
            code: Some(code),
            message,
        });
        if let Self::RateLimited(Some(secs)) = self
            && let Ok(v) = axum::http::HeaderValue::from_str(&secs.to_string())
        {
            resp.headers_mut().insert("retry-after", v);
        }
        resp
    }
}

pub async fn api_not_found() -> AppError {
    AppError::NotFound
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::auth_store::AuthStoreError;
    use crate::services::edited_thumb::EditedThumbError;
    use crate::services::instance_store::InstanceStoreError;
    #[cfg(feature = "ml")]
    use crate::services::model_store::ModelStoreError;

    fn status_of(err: AppError) -> StatusCode {
        err.parts().0
    }

    #[test]
    fn render_error_maps_to_status() {
        let cases = [
            (
                RenderError::Pipeline(PipelineError::Cancelled),
                StatusCode::CONFLICT,
            ),
            (
                RenderError::Pipeline(PipelineError::Unsupported("nef".into())),
                StatusCode::UNPROCESSABLE_ENTITY,
            ),
            (RenderError::Lut("bad".into()), StatusCode::BAD_REQUEST),
            (
                RenderError::Upstream(ImmichError::NotFound),
                StatusCode::NOT_FOUND,
            ),
        ];
        for (err, expected) in cases {
            assert_eq!(status_of(err.into()), expected);
        }
    }

    #[cfg(feature = "ml")]
    #[test]
    fn model_store_errors_map_to_status() {
        assert_eq!(
            status_of(ModelStoreError::NotFound.into()),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            status_of(ModelStoreError::Invalid("nope".into()).into()),
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn store_errors_map_to_status() {
        assert_eq!(
            status_of(EditedThumbError::HashMismatch.into()),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            status_of(AuthStoreError::AlreadyConfigured.into()),
            StatusCode::CONFLICT
        );
        assert_eq!(
            status_of(InstanceStoreError::AlreadyConfigured.into()),
            StatusCode::CONFLICT
        );
    }
}
