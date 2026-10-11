use std::sync::Arc;

use raw_pipeline::dcp::DcpProfile;
use raw_pipeline::edits::{Edits, WhiteBalance};
use raw_pipeline::frame::RawFrame;
use raw_pipeline::white_balance::{auto_white_balance, sample_white_balance};
use serde::Deserialize;

use crate::error::AppError;

#[derive(Debug, Deserialize)]
pub struct SamplePoint {
    pub u: f32,
    pub v: f32,
    #[serde(default)]
    pub edits: Edits,
}

impl SamplePoint {
    pub fn validate(&self) -> Result<(), AppError> {
        if (0.0..=1.0).contains(&self.u) && (0.0..=1.0).contains(&self.v) {
            return Ok(());
        }
        Err(AppError::BadRequest(
            "sample point must be inside 0..1".to_string(),
        ))
    }
}

pub async fn sample(
    frame: Arc<RawFrame>,
    point: SamplePoint,
    profile: Option<Arc<DcpProfile>>,
) -> Result<WhiteBalance, AppError> {
    solve(
        move || sample_white_balance(&frame, &point.edits, profile.as_deref(), point.u, point.v),
        "No usable colour here",
    )
    .await
}

pub async fn auto(
    frame: Arc<RawFrame>,
    edits: Edits,
    profile: Option<Arc<DcpProfile>>,
) -> Result<WhiteBalance, AppError> {
    solve(
        move || auto_white_balance(&frame, &edits, profile.as_deref()),
        "No neutral colour found",
    )
    .await
}

async fn solve(
    solver: impl FnOnce() -> Option<WhiteBalance> + Send + 'static,
    unsolved: &str,
) -> Result<WhiteBalance, AppError> {
    let solved = tokio::task::spawn_blocking(solver)
        .await
        .map_err(|e| AppError::internal("white balance task", &e))?;
    solved.ok_or_else(|| AppError::Unprocessable(unsolved.to_string()))
}

#[cfg(test)]
mod tests;
