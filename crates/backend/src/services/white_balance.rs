use std::sync::Arc;

use raw_pipeline::dcp::DcpProfile;
use raw_pipeline::edits::{Edits, WhiteBalance};
use raw_pipeline::frame::RawFrame;
use raw_pipeline::white_balance::{
    SampleArea, auto_white_balance, sample_white_balance, sample_white_balance_area,
};
use serde::Deserialize;

use crate::error::AppError;

const EDGE_SLACK: f32 = 1e-4;

#[derive(Debug, Deserialize)]
pub struct WhiteBalanceSample {
    pub u: f32,
    pub v: f32,
    #[serde(default)]
    pub w: f32,
    #[serde(default)]
    pub h: f32,
    #[serde(default)]
    pub edits: Edits,
}

impl WhiteBalanceSample {
    pub fn validate(&self) -> Result<(), AppError> {
        let unit = 0.0..=1.0;
        let extent = 0.0..=1.0 + EDGE_SLACK;
        if unit.contains(&self.u)
            && unit.contains(&self.v)
            && self.w >= 0.0
            && self.h >= 0.0
            && extent.contains(&(self.u + self.w))
            && extent.contains(&(self.v + self.h))
        {
            return Ok(());
        }
        Err(AppError::BadRequest(
            "white balance sample must be inside 0..1".to_string(),
        ))
    }
}

pub async fn sample(
    frame: Arc<RawFrame>,
    req: WhiteBalanceSample,
    profile: Option<Arc<DcpProfile>>,
) -> Result<WhiteBalance, AppError> {
    solve(
        move || {
            let profile = profile.as_deref();
            if req.w > 0.0 || req.h > 0.0 {
                let area = SampleArea {
                    u: req.u,
                    v: req.v,
                    w: req.w,
                    h: req.h,
                };
                return sample_white_balance_area(&frame, &req.edits, profile, area);
            }
            sample_white_balance(&frame, &req.edits, profile, req.u, req.v)
        },
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
