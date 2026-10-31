use std::error::Error;
use std::sync::Arc;

use raw_pipeline::PipelineError;

use crate::immich::ImmichError;
use crate::services::model_store::ModelStoreError;
use crate::services::render::RenderError;

pub const BUSY_RETRY_SECS: u64 = 30;

#[derive(Debug, Clone, thiserror::Error)]
pub enum DenoiseError {
    #[error("unsupported: {0}")]
    Unsupported(String),
    #[error("too large: {0}")]
    TooLarge(String),
    #[error("upstream: {0}")]
    Upstream(#[from] ImmichError),
    #[error("{context}")]
    Internal {
        context: &'static str,
        #[source]
        cause: Arc<dyn Error + Send + Sync>,
    },
    #[error("the AI denoise cache was purged during the bake")]
    Purged,
    #[error("the AI denoise bake did not finish in time")]
    StillRunning,
    #[error("too many AI denoise bakes are queued for this user")]
    Busy,
}

impl DenoiseError {
    pub fn internal(context: &'static str, cause: impl Error + Send + Sync + 'static) -> Self {
        Self::Internal {
            context,
            cause: Arc::new(cause),
        }
    }

    pub fn stopped(context: &'static str) -> Self {
        Self::internal(context, std::io::Error::other("the denoise worker stopped"))
    }

    pub fn user_message(&self) -> &'static str {
        match self {
            Self::Unsupported(_) => "AI denoise needs a RAW photo with usable color data.",
            Self::TooLarge(_) => "This photo is too large for the server's memory.",
            Self::Upstream(_) => "The photo could not be downloaded from Immich.",
            Self::Internal { .. } => "Denoising failed. Check the server log.",
            Self::Purged => "The AI denoise cache was cleared. Try again.",
            Self::StillRunning => "AI denoise is still running. Try again when it finishes.",
            Self::Busy => "Too many photos are waiting for AI denoise. Try again shortly.",
        }
    }
}

impl From<PipelineError> for DenoiseError {
    fn from(err: PipelineError) -> Self {
        match err {
            PipelineError::Unsupported(reason) => Self::Unsupported(reason),
            PipelineError::OverBudget(reason) => Self::TooLarge(reason),
            other => Self::internal("baking the denoise plane", other),
        }
    }
}

impl From<RenderError> for DenoiseError {
    fn from(err: RenderError) -> Self {
        match err {
            RenderError::Upstream(e) => Self::Upstream(e),
            RenderError::Pipeline(e) => e.into(),
            other => Self::internal("loading the photo for AI denoise", other),
        }
    }
}

impl From<ModelStoreError> for DenoiseError {
    fn from(err: ModelStoreError) -> Self {
        Self::internal("resolving the denoise model", err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_messages_hide_the_cause() {
        let internal = DenoiseError::internal("x", std::io::Error::other("/secret/path"));
        assert_eq!(
            internal.user_message(),
            "Denoising failed. Check the server log."
        );
        let upstream: DenoiseError = RenderError::Upstream(ImmichError::Timeout).into();
        assert!(matches!(upstream, DenoiseError::Upstream(_)));
        assert_eq!(
            upstream.user_message(),
            "The photo could not be downloaded from Immich."
        );
        let unsupported: DenoiseError =
            RenderError::Pipeline(PipelineError::Unsupported("jpeg".into())).into();
        assert!(matches!(unsupported, DenoiseError::Unsupported(_)));
    }

    #[test]
    fn a_photo_over_the_memory_budget_is_too_large_not_internal() {
        let refused = PipelineError::OverBudget("decoding this photo needs 9000 MiB".into());
        let error: DenoiseError = RenderError::Pipeline(refused).into();
        if !matches!(&error, DenoiseError::TooLarge(reason) if reason.contains("9000 MiB")) {
            panic!("an over-budget photo became {error:?}");
        }
    }
}
