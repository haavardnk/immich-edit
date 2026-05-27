use axum::Json;
use axum::extract::State;
use serde::Serialize;
use serde_json::{Value, json};

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct Health {
    pub status: &'static str,
    pub version: &'static str,
    pub renderer_mode: &'static str,
    pub renderer_active: &'static str,
    pub gpu_adapter: Option<String>,
    pub immich_reachable: bool,
    pub db_ready: bool,
    pub db_migration_version: Option<i64>,
    pub config: crate::config::RedactedConfig,
}

pub async fn live() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

pub async fn health(State(state): State<AppState>) -> Json<Health> {
    let immich_reachable = state.immich.ping().await.is_ok();
    let db_ready = state.edits.ready().await.is_ok();
    let db_migration_version = state.edits.migration_version().await.ok().flatten();
    Json(Health {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        renderer_mode: state.config.renderer.as_str(),
        renderer_active: state.render.active().as_str(),
        gpu_adapter: state.render.gpu_label().map(|s| s.to_string()),
        immich_reachable,
        db_ready,
        db_migration_version,
        config: state.config.redacted(),
    })
}
