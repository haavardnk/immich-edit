use axum::Json;
use axum::extract::State;

use crate::error::AppError;
use crate::routes::auth::AuthCtx;
use crate::services::export::{ExportOutputsQuery, ExportOutputsSummary, find_export_outputs};
use crate::state::AppState;

pub async fn query(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Json(body): Json<ExportOutputsQuery>,
) -> Result<Json<ExportOutputsSummary>, AppError> {
    let lookup = body.parse()?;
    let summary = find_export_outputs(&state, &ctx.immich, ctx.owner, &lookup).await?;
    Ok(Json(summary))
}
