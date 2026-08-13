use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

pub async fn purge_owner(state: &AppState, owner: Uuid) -> Result<(), AppError> {
    state.queue.cancel_owner(owner).await;
    state.jobs.purge_owner(owner).await?;
    state.edits.purge_owner(owner).await?;
    state.auth.revoke_all_for_user(owner).await?;
    state.rasters.purge_owner(owner).await?;
    state.edited_thumb.purge_owner(owner).await?;
    crate::services::export::purge_owner_exports(state, owner).await;
    Ok(())
}

pub async fn purge_instance(state: &AppState) {
    state.queue.cancel_all().await;
    state.render.clear_frame_caches().await;
    state.preview_meta.clear().await;
    state.tag_counts.clear().await;
    state.people_counts.clear().await;
    if let Err(error) = state.rasters.purge_all().await {
        tracing::warn!(error = %error, "purge rasters after rebind");
    }
    if let Err(error) = state.edited_thumb.purge_all().await {
        tracing::warn!(error = %error, "purge edited thumbnails after rebind");
    }
    crate::services::export::purge_all_exports(state).await;
}
