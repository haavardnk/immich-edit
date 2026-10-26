use uuid::Uuid;

use crate::error::AppError;
use crate::immich::ImmichClient;
use crate::services::render::RenderIdentity;
use crate::state::AppState;

pub async fn ensure_enabled(immich: &ImmichClient) -> Result<(), AppError> {
    if immich.server_features().await?.trash {
        Ok(())
    } else {
        Err(AppError::TrashDisabled)
    }
}

pub async fn move_to_trash(
    state: &AppState,
    identity: RenderIdentity,
    immich: &ImmichClient,
    ids: &[Uuid],
) -> Result<(), AppError> {
    ensure_enabled(immich).await?;
    discard(state, identity, immich, ids).await
}

pub async fn discard(
    state: &AppState,
    identity: RenderIdentity,
    immich: &ImmichClient,
    ids: &[Uuid],
) -> Result<(), AppError> {
    immich.trash_assets(ids).await?;
    forget_counts(state, identity).await;
    Ok(())
}

pub async fn restore(
    state: &AppState,
    identity: RenderIdentity,
    immich: &ImmichClient,
    ids: &[Uuid],
) -> Result<u64, AppError> {
    let count = immich.restore_assets(ids).await?;
    forget_counts(state, identity).await;
    Ok(count)
}

async fn forget_counts(state: &AppState, identity: RenderIdentity) {
    state
        .tag_counts
        .clear_tenant(identity.owner, identity.server_epoch)
        .await;
    state
        .people_counts
        .clear_tenant(identity.owner, identity.server_epoch)
        .await;
}
