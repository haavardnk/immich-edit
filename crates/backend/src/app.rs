use axum::{Router, routing::get};
use axum::response::Json;
use serde_json::{json, Value};

use crate::state::AppState;

async fn health() -> Json<Value> {
    Json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "renderer": "cpu",
        "status": "ok"
    }))
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .with_state(state)
}
