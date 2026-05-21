use axum::Json;
use axum::extract::State;
use serde::Serialize;

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct Health {
    pub status: &'static str,
    pub version: &'static str,
    pub renderer: &'static str,
    pub immich_reachable: bool,
    pub config: crate::config::RedactedConfig,
}

pub async fn health(State(state): State<AppState>) -> Json<Health> {
    let immich_reachable = state.immich.ping().await.is_ok();
    Json(Health {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        renderer: state.config.renderer.as_str(),
        immich_reachable,
        config: state.config.redacted(),
    })
}
