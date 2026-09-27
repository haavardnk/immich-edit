use std::sync::Arc;

use raw_pipeline::edits::Edits;
use raw_pipeline::frame::RawFrame;
use raw_pipeline::white_balance::{auto_white_balance, sample_white_balance};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct WhiteBalance {
    pub wb_temp: f64,
    pub wb_tint: f64,
}

pub async fn sample(frame: Arc<RawFrame>, point: SamplePoint) -> Result<WhiteBalance, AppError> {
    solve(
        move || sample_white_balance(&frame, &point.edits, point.u, point.v),
        "No usable colour here",
    )
    .await
}

pub async fn auto(frame: Arc<RawFrame>, edits: Edits) -> Result<WhiteBalance, AppError> {
    solve(
        move || auto_white_balance(&frame, &edits),
        "No neutral colour found",
    )
    .await
}

async fn solve(
    solver: impl FnOnce() -> Option<(f64, f64)> + Send + 'static,
    unsolved: &str,
) -> Result<WhiteBalance, AppError> {
    let solved = tokio::task::spawn_blocking(solver)
        .await
        .map_err(|_| AppError::Internal)?;
    let Some((wb_temp, wb_tint)) = solved else {
        return Err(AppError::Unprocessable(unsolved.to_string()));
    };
    Ok(WhiteBalance { wb_temp, wb_tint })
}

#[cfg(test)]
mod tests;
