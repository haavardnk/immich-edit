use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use segment::catalog;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::error::AppError;
use crate::routes::auth::{AdminCtx, AuthCtx};
use crate::services::model_store::ModelStoreError;
use crate::state::AppState;

#[derive(Serialize)]
pub struct CatalogView {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: &'static str,
    pub tier: &'static str,
    pub license: &'static str,
    pub source: &'static str,
    pub notes: &'static str,
    pub size_bytes: u64,
    pub input_edge: u32,
    pub gpu_ms: u32,
    pub gpu_mb: u32,
    pub cpu_ms: u32,
    pub cpu_mb: u32,
    pub installed: bool,
    pub installing: bool,
    pub progress_bytes: u64,
    pub install_error: Option<String>,
}

#[derive(Serialize)]
pub struct SemanticClassView {
    pub id: &'static str,
    pub name: &'static str,
}

#[derive(Serialize)]
pub struct ModelsResponse {
    pub runtime: &'static str,
    pub enabled: bool,
    pub models: Vec<CatalogView>,
    pub active: BTreeMap<&'static str, String>,
    pub semantic_classes: Vec<SemanticClassView>,
}

#[derive(Deserialize)]
pub struct SelectBody {
    pub kind: String,
    pub model_id: String,
}

fn tier_name(tier: segment::Tier) -> &'static str {
    match tier {
        segment::Tier::Recommended => "recommended",
        segment::Tier::Alternative => "alternative",
        segment::Tier::LowMemory => "low_memory",
    }
}

async fn catalog_view(state: &AppState) -> Result<Vec<CatalogView>, AppError> {
    let installs = state.installs.snapshot();
    let mut out = Vec::with_capacity(catalog::CATALOG.len());
    for entry in catalog::CATALOG {
        let installed = state
            .models
            .find_by_catalog(entry.id)
            .await
            .map_err(|_| AppError::Internal)?
            .is_some();
        let install = installs.get(entry.id);
        out.push(CatalogView {
            id: entry.id,
            name: entry.name,
            kind: entry.kind.as_str(),
            tier: tier_name(entry.tier),
            license: entry.license,
            source: entry.source,
            notes: entry.notes,
            size_bytes: entry.total_bytes(),
            input_edge: entry.spec.input_edge,
            gpu_ms: entry.cost.gpu_ms,
            gpu_mb: entry.cost.gpu_mb,
            cpu_ms: entry.cost.cpu_ms,
            cpu_mb: entry.cost.cpu_mb,
            installed,
            installing: install.is_some_and(|i| i.error.is_none()),
            progress_bytes: install.map_or(0, |i| i.received),
            install_error: install.and_then(|i| i.error.clone()),
        });
    }
    Ok(out)
}

pub async fn list(
    State(state): State<AppState>,
    _ctx: AuthCtx,
) -> Result<Json<ModelsResponse>, AppError> {
    let models = catalog_view(&state).await?;
    let mut active = BTreeMap::new();
    for kind in catalog::KINDS {
        if let Some(id) = state.segment.active_model(*kind).await {
            active.insert(kind.as_str(), id);
        }
    }
    Ok(Json(ModelsResponse {
        runtime: state.config.segment_runtime.as_str(),
        enabled: state.segment.enabled(),
        models,
        active,
        semantic_classes: catalog::SEMANTIC_CLASSES
            .iter()
            .map(|c| SemanticClassView {
                id: c.id,
                name: c.name,
            })
            .collect(),
    }))
}

pub async fn select(
    State(state): State<AppState>,
    _admin: AdminCtx,
    Json(body): Json<SelectBody>,
) -> Result<StatusCode, AppError> {
    let entry = catalog::find(&body.model_id).ok_or(AppError::NotFound)?;
    if entry.kind.as_str() != body.kind {
        return Err(AppError::BadRequest(format!(
            "{} is not a {} model",
            entry.id, body.kind
        )));
    }
    state
        .models
        .set_preferred(&body.kind, entry.id)
        .await
        .map_err(|e| match e {
            ModelStoreError::NotFound => AppError::BadRequest("model is not installed".into()),
            _ => AppError::Internal,
        })?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn install(
    State(state): State<AppState>,
    _admin: AdminCtx,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let entry = catalog::find(&id).ok_or(AppError::NotFound)?;
    if state
        .models
        .find_by_catalog(entry.id)
        .await
        .map_err(|_| AppError::Internal)?
        .is_some()
    {
        return Ok((
            StatusCode::OK,
            Json(serde_json::json!({ "id": entry.id, "installed": true })),
        ));
    }

    state.installs.start(entry);

    Ok((
        StatusCode::ACCEPTED,
        Json(serde_json::json!({ "id": entry.id, "installing": true })),
    ))
}

pub async fn remove(
    State(state): State<AppState>,
    _admin: AdminCtx,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    state.models.remove(&id).await.map_err(|e| match e {
        ModelStoreError::NotFound => AppError::NotFound,
        _ => AppError::Internal,
    })?;
    Ok(StatusCode::NO_CONTENT)
}
