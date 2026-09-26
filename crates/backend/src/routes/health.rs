use axum::Json;
use axum::extract::State;
use serde::Serialize;
use serde_json::{Value, json};

use crate::host::{HostInfo, host_info};
use crate::immich::ImmichConnectionStatus;
use crate::immich::dto::ServerVersion;
use crate::routes::auth::AuthCtx;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct Health {
    pub status: &'static str,
    pub version: &'static str,
    pub renderer_mode: &'static str,
    pub renderer_active: &'static str,
    pub gpu_adapter: Option<String>,
    pub gpu_software: bool,
    pub host: &'static HostInfo,
    pub heif_codecs: raw_pipeline::codecs::HeifCodecs,
    pub immich_reachable: bool,
    pub immich_status: ImmichConnectionStatus,
    pub db_ready: bool,
    pub db_migration_version: Option<i64>,
    pub config: crate::config::RedactedConfig,
}

pub async fn live() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

pub async fn health(State(state): State<AppState>, ctx: AuthCtx) -> Json<Health> {
    let immich_status = ImmichConnectionStatus::from_ping(ctx.immich.ping().await);
    let db_ready = state.edits.ready().await.is_ok();
    let db_migration_version = state.edits.migration_version().await.ok().flatten();
    Json(Health {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        renderer_mode: state.config.renderer.as_str(),
        renderer_active: state.render.active().as_str(),
        gpu_adapter: state.render.gpu_label().map(|s| s.to_string()),
        gpu_software: state.render.software_gpu(),
        host: host_info(),
        heif_codecs: raw_pipeline::codecs::heif_codecs(),
        immich_reachable: immich_status.ok,
        immich_status,
        db_ready,
        db_migration_version,
        config: state.config.redacted(),
    })
}

const MIN_RATING_FILTER: ServerVersion = ServerVersion {
    major: 3,
    minor: 2,
    patch: 0,
};

#[derive(Debug, Serialize)]
pub struct ImmichCapabilities {
    pub immich_version: Option<String>,
    pub min_rating_filter: bool,
}

pub async fn immich_capabilities(ctx: AuthCtx) -> Json<ImmichCapabilities> {
    let version = match ctx.immich.server_version().await {
        Ok(version) => Some(version),
        Err(e) => {
            tracing::warn!(error = %e, "immich server version unavailable");
            None
        }
    };
    Json(ImmichCapabilities {
        immich_version: version.map(|v| format!("{}.{}.{}", v.major, v.minor, v.patch)),
        min_rating_filter: version.is_some_and(|v| v >= MIN_RATING_FILTER),
    })
}
