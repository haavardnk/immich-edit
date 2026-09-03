use std::time::Duration;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::StatusCode;
use axum::http::header::{HeaderName, HeaderValue};
use axum::middleware::{from_fn, from_fn_with_state};
use axum::routing::{get, patch, post, put};
use tower::ServiceBuilder;
use tower_governor::GovernorLayer;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::GlobalKeyExtractor;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::compression::CompressionLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::request_id::{
    MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer,
};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use crate::error::api_not_found;
use crate::routes;
use crate::state::AppState;

mod cors;
mod middleware;

use cors::build_cors;
use middleware::{
    auth_middleware, csrf_guard, inject_auth_context, request_id_scope, resolve_client_meta,
};

const REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

#[derive(Clone, Default)]
struct UuidRequestId;

impl MakeRequestId for UuidRequestId {
    fn make_request_id<B>(&mut self, _req: &http::Request<B>) -> Option<RequestId> {
        let id = Uuid::new_v4().to_string();
        HeaderValue::from_str(&id).ok().map(RequestId::new)
    }
}

#[cfg(feature = "ml")]
fn model_routes() -> Router<AppState> {
    Router::new()
        .route("/masks/models", get(routes::models::list))
        .route("/assets/{id}/masks/generate", post(routes::masks::generate))
        .route("/masks/rebake", post(routes::masks::rebake))
        .route("/assets/{id}/masks/click", post(routes::masks::click))
        .route("/admin/masks/default", put(routes::models::select))
        .route(
            "/admin/models/{id}",
            post(routes::models::install).delete(routes::models::remove),
        )
}

#[cfg(not(feature = "ml"))]
fn model_routes() -> Router<AppState> {
    Router::new()
}

pub fn router(state: AppState) -> Router {
    let heavy_cfg = std::sync::Arc::new(
        GovernorConfigBuilder::default()
            .key_extractor(GlobalKeyExtractor)
            .per_millisecond(2000)
            .burst_size(20)
            .finish()
            .expect("heavy governor config"),
    );
    let heavy = GovernorLayer::new(heavy_cfg);

    let request_timeout = Duration::from_secs(state.config.request_timeout_secs);
    let export_timeout = Duration::from_secs(
        state
            .config
            .original_timeout_secs
            .saturating_add(state.config.export_timeout_secs),
    );

    let exports = Router::new()
        .route(
            "/assets/{id}/export",
            get(routes::export::get_export).post(routes::export::post_export),
        )
        .route(
            "/assets/{id}/export/immich",
            post(routes::export::post_export_immich),
        )
        .layer(heavy)
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            export_timeout,
        ));

    let api = Router::new()
        .route("/health", get(routes::health::health))
        .route("/health/live", get(routes::health::live))
        .route("/auth/login/password", post(routes::auth::login_password))
        .route("/auth/login/api-key", post(routes::auth::login_api_key))
        .route("/auth/logout", post(routes::auth::logout_session))
        .route("/auth/me", get(routes::auth::me))
        .route("/auth/sessions", get(routes::auth::list_sessions))
        .route(
            "/auth/sessions/{id}",
            axum::routing::delete(routes::auth::revoke_session),
        )
        .route(
            "/auth/sessions/revoke-all",
            post(routes::auth::revoke_all_sessions),
        )
        .route("/admin/users", get(routes::admin::list_users))
        .route("/admin/users/{id}/access", put(routes::admin::set_access))
        .route(
            "/admin/users/{id}/data",
            axum::routing::delete(routes::admin::purge_user_data),
        )
        .route("/admin/instance", get(routes::admin::instance_info))
        .route("/admin/instance/rebind", post(routes::admin::rebind))
        .route("/setup/status", get(routes::setup::status))
        .route("/setup/complete", post(routes::setup::complete))
        .route("/debug/timings", get(routes::debug::timings))
        .route("/albums", get(routes::albums::list))
        .route("/albums/{id}", get(routes::albums::detail))
        .route("/people", get(routes::people::list))
        .route("/people/{id}/thumb", get(routes::people::thumbnail))
        .route("/tags", get(routes::tags::list).put(routes::tags::upsert))
        .route(
            "/tags/{tag_id}/assets/{asset_id}",
            put(routes::tags::tag_asset).delete(routes::tags::untag_asset),
        )
        .route("/folders/paths", get(routes::folders::paths))
        .route("/folders/assets", get(routes::folders::assets))
        .route("/search/metadata", post(routes::search::metadata))
        .route("/search/smart", post(routes::search::smart))
        .route("/search/statistics", post(routes::search::statistics))
        .route("/edits", get(routes::edits::list))
        .route(
            "/presets",
            get(routes::presets::list).post(routes::presets::create),
        )
        .route(
            "/presets/{id}",
            get(routes::presets::get)
                .put(routes::presets::update)
                .delete(routes::presets::delete),
        )
        .route(
            "/jobs",
            get(routes::jobs::list)
                .post(routes::jobs::create)
                .delete(routes::jobs::clear),
        )
        .route("/jobs/{id}", get(routes::jobs::get))
        .route("/jobs/{id}/cancel", post(routes::jobs::cancel))
        .route("/jobs/{id}/download", get(routes::jobs::download))
        .route("/jobs/{id}/events", get(routes::jobs::events))
        .route(
            "/assets/{id}",
            get(routes::assets::detail).put(routes::assets::update),
        )
        .route("/assets/{id}/thumb", get(routes::assets::thumbnail))
        .route(
            "/assets/{id}/copies",
            get(routes::copies::list).post(routes::copies::create),
        )
        .route(
            "/copies/{id}",
            patch(routes::copies::rename).delete(routes::copies::delete),
        )
        .route("/assets/{id}/edited-thumb", get(routes::edited_thumb::get))
        .route("/assets/{id}/faces", get(routes::faces::list))
        .route(
            "/assets/{id}/edits",
            get(routes::edits::get)
                .put(routes::edits::put)
                .delete(routes::edits::delete),
        )
        .route("/assets/{id}/edits/auto", post(routes::edits::auto))
        .route("/assets/{id}/edits/history", get(routes::edits::history))
        .route("/assets/{id}/edits/restore", post(routes::edits::restore))
        .route(
            "/assets/{id}/lens-profile",
            get(routes::lens_profile::get_lens_profile),
        )
        .route(
            "/assets/{id}/preview",
            get(routes::preview::get_preview).post(routes::preview::post_preview),
        )
        .route(
            "/assets/{id}/preview/meta/{meta_id}",
            get(routes::preview::get_meta),
        )
        .route("/rasters", post(routes::rasters::upload))
        .route("/rasters/{raster_id}", get(routes::rasters::get))
        .route("/rasters/{raster_id}/meta", get(routes::rasters::meta))
        .merge(model_routes())
        .merge(
            Router::new()
                .route("/luts", get(routes::luts::list).post(routes::luts::import))
                .route("/luts/{id}", axum::routing::delete(routes::luts::delete))
                .route("/dcp", get(routes::dcp::list).post(routes::dcp::import))
                .route("/dcp/match", get(routes::dcp::match_camera))
                .route("/dcp/{id}", axum::routing::delete(routes::dcp::delete))
                .layer(DefaultBodyLimit::max(32 * 1024 * 1024)),
        )
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            request_timeout,
        ))
        .merge(exports)
        .fallback(api_not_found)
        .layer(from_fn(auth_middleware))
        .layer(from_fn_with_state(state.clone(), inject_auth_context))
        .layer(from_fn_with_state(state.clone(), csrf_guard))
        .layer(from_fn(resolve_client_meta))
        .layer(from_fn(request_id_scope));

    let web_dir = std::env::var("WEB_DIR").unwrap_or_else(|_| "./web".into());
    let fallback_file = format!("{web_dir}/200.html");
    let has_web = std::path::Path::new(&fallback_file).exists();

    let mut root = Router::new().nest("/api", api);
    if has_web {
        let spa = ServeDir::new(&web_dir).fallback(ServeFile::new(&fallback_file));
        root = root.fallback_service(spa);
    }

    let body_bytes = (state.config.max_body_mb as usize).saturating_mul(1024 * 1024);
    let cors = build_cors(&state.config.allowed_origins);

    root.with_state(state).layer(
        ServiceBuilder::new()
            .layer(SetRequestIdLayer::new(
                REQUEST_ID_HEADER.clone(),
                UuidRequestId,
            ))
            .layer(PropagateRequestIdLayer::new(REQUEST_ID_HEADER.clone()))
            .layer(TraceLayer::new_for_http())
            .layer(CatchPanicLayer::new())
            .layer(SetResponseHeaderLayer::if_not_present(
                HeaderName::from_static("x-content-type-options"),
                HeaderValue::from_static("nosniff"),
            ))
            .layer(SetResponseHeaderLayer::if_not_present(
                HeaderName::from_static("referrer-policy"),
                HeaderValue::from_static("no-referrer"),
            ))
            .layer(SetResponseHeaderLayer::if_not_present(
                HeaderName::from_static("x-frame-options"),
                HeaderValue::from_static("DENY"),
            ))
            .layer(CompressionLayer::new())
            .layer(DefaultBodyLimit::max(body_bytes))
            .layer(RequestBodyLimitLayer::new(body_bytes))
            .layer(TimeoutLayer::with_status_code(
                StatusCode::REQUEST_TIMEOUT,
                request_timeout.max(export_timeout),
            ))
            .layer(cors),
    )
}
