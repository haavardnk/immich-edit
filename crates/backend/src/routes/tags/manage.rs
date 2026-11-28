use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::immich::ImmichError;
use crate::immich::dto::{NewTag, TagPatch, TagSummary};
use crate::patch_field::present;
use crate::routes::auth::AuthCtx;
use crate::state::AppState;
use crate::tag_rules::{MANAGED_ROOT, is_managed, sibling_named, tag_color, tag_name};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateTagBody {
    name: String,
    #[serde(rename = "parentId", default)]
    parent_id: Option<Uuid>,
    #[serde(default)]
    color: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateTagBody {
    #[serde(default)]
    name: Option<String>,
    #[serde(default, deserialize_with = "present")]
    color: Option<Option<String>>,
}

impl TryFrom<UpdateTagBody> for TagPatch {
    type Error = AppError;

    fn try_from(body: UpdateTagBody) -> Result<Self, AppError> {
        let name = body
            .name
            .as_deref()
            .map(tag_name)
            .transpose()
            .map_err(AppError::BadRequest)?;
        let color = body
            .color
            .map(|color| color.as_deref().map(tag_color).transpose())
            .transpose()
            .map_err(AppError::BadRequest)?;
        let patch = Self { name, color };
        if patch.is_empty() {
            return Err(AppError::BadRequest("nothing to update".into()));
        }
        Ok(patch)
    }
}

fn managed() -> AppError {
    AppError::BadRequest(format!("{MANAGED_ROOT} tags are managed by immich-edit"))
}

fn editable(
    tags: &[TagSummary],
    id: Uuid,
    missing: impl FnOnce() -> AppError,
) -> Result<&TagSummary, AppError> {
    let tag = tags.iter().find(|tag| tag.id == id).ok_or_else(missing)?;
    if is_managed(&tag.value) {
        return Err(managed());
    }
    Ok(tag)
}

fn ensure_free(
    tags: &[TagSummary],
    parent_id: Option<Uuid>,
    name: &str,
    except: Option<Uuid>,
) -> Result<(), AppError> {
    if parent_id.is_none() && is_managed(name) {
        return Err(managed());
    }
    let Some(clash) = sibling_named(tags, parent_id, name, except) else {
        return Ok(());
    };
    let path = clash.value.split('/').collect::<Vec<_>>().join(" / ");
    Err(AppError::Conflict(format!("{path} already exists")))
}

fn duplicate_as_conflict(err: ImmichError) -> AppError {
    match err {
        ImmichError::Rejected {
            status: 400,
            message,
        } if message.contains("already exists") => AppError::Conflict(message),
        other => other.into(),
    }
}

pub async fn create(
    ctx: AuthCtx,
    Json(body): Json<CreateTagBody>,
) -> Result<Json<TagSummary>, AppError> {
    let name = tag_name(&body.name).map_err(AppError::BadRequest)?;
    let color = body
        .color
        .as_deref()
        .map(tag_color)
        .transpose()
        .map_err(AppError::BadRequest)?;
    let tags = ctx.immich.list_tags().await?;
    if let Some(parent) = body.parent_id {
        editable(&tags, parent, || {
            AppError::BadRequest("unknown parent tag".into())
        })?;
    }
    ensure_free(&tags, body.parent_id, &name, None)?;
    let new_tag = NewTag {
        name,
        parent_id: body.parent_id,
        color,
    };
    let mut created = ctx
        .immich
        .create_tag(&new_tag)
        .await
        .map_err(duplicate_as_conflict)?;
    created.asset_count = Some(0);
    Ok(Json(created))
}

pub async fn update(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateTagBody>,
) -> Result<Json<TagSummary>, AppError> {
    let patch = TagPatch::try_from(body)?;
    let tags = ctx.immich.list_tags().await?;
    let tag = editable(&tags, id, || AppError::NotFound)?;
    if let Some(name) = &patch.name {
        let caps = state.capabilities.get(ctx.server_epoch, &ctx.immich).await;
        if !caps.tag_rename {
            return Err(AppError::BadRequest(
                "renaming tags needs Immich 3.3 or later".into(),
            ));
        }
        ensure_free(&tags, tag.parent_id, name, Some(id))?;
    }
    let updated = ctx.immich.update_tag(id, &patch).await?;
    state.search_caches.names_changed();
    Ok(Json(updated))
}

pub async fn delete(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let tags = ctx.immich.list_tags().await?;
    editable(&tags, id, || AppError::NotFound)?;
    ctx.immich.delete_tag(id).await?;
    state
        .tag_counts
        .clear_tenant(ctx.owner, ctx.server_epoch)
        .await;
    state.search_caches.membership_changed();
    Ok(StatusCode::NO_CONTENT)
}
