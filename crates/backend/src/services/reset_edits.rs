use uuid::Uuid;

use crate::services::job_runner::ItemOutcome;
use crate::services::job_store::JobRecord;
use crate::state::AppState;

pub const RESET_EDITS_KIND: &str = "reset_edits";

pub async fn run_reset_edits_item(
    state: &AppState,
    _job: &JobRecord,
    asset_id: Uuid,
) -> ItemOutcome {
    let deleted = state
        .edits
        .delete(asset_id, Some("Bulk reset"))
        .await
        .map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "deleted": deleted }))
}
