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
use tower_http::compression::predicate::{DefaultPredicate, NotForContentType, Predicate};
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
    auth_middleware, count_timeouts, csrf_guard, inject_auth_context, request_id_scope,
    resolve_client_meta,
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
        .route(
            "/admin/models/{id}/install",
            axum::routing::delete(routes::models::cancel_install),
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
        ))
        .layer(from_fn_with_state(state.clone(), count_timeouts));

    let api = Router::new()
        .route("/health", get(routes::health::health))
        .route(
            "/immich/capabilities",
            get(routes::health::immich_capabilities),
        )
        .route("/health/live", get(routes::health::live))
        .route("/auth/login/password", post(routes::auth::login_password))
        .route("/auth/login/api-key", post(routes::auth::login_api_key))
        .route("/auth/providers", get(routes::auth::oauth::providers))
        .route("/auth/oauth/start", post(routes::auth::oauth::start))
        .route("/auth/oauth/callback", post(routes::auth::oauth::callback))
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
        .route("/setup/providers", get(routes::setup::oauth::providers))
        .route("/setup/oauth/start", post(routes::setup::oauth::start))
        .route(
            "/setup/oauth/complete",
            post(routes::setup::oauth::complete),
        )
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
        .route("/search/window", post(routes::search::window))
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
            "/assets/{id}/edits/white-balance",
            post(routes::edits::white_balance_sample),
        )
        .route(
            "/assets/{id}/edits/white-balance/auto",
            post(routes::edits::white_balance_auto),
        )
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
        .route(
            "/assets/{id}/preview/meta/{meta_id}/scope/{kind}",
            get(routes::preview::get_scope),
        )
        .route("/assets/{id}/source", post(routes::source::post_source))
        .route("/rasters", post(routes::rasters::upload))
        .route("/rasters/{raster_id}", get(routes::rasters::get))
        .route("/rasters/{raster_id}/meta", get(routes::rasters::meta))
        .merge(model_routes())
        .merge(
            Router::new()
                .route("/luts", get(routes::luts::list).post(routes::luts::import))
                .route("/luts/{id}", axum::routing::delete(routes::luts::delete))
                .route("/luts/{id}/cube", get(routes::luts::cube))
                .route("/dcp", get(routes::dcp::list).post(routes::dcp::import))
                .route("/dcp/match", get(routes::dcp::match_camera))
                .route("/dcp/{id}", axum::routing::delete(routes::dcp::delete))
                .route("/dcp/{id}/raw", get(routes::dcp::raw))
                .route(
                    "/watermarks",
                    get(routes::watermarks::list).post(routes::watermarks::import),
                )
                .route(
                    "/watermarks/{id}",
                    axum::routing::delete(routes::watermarks::delete),
                )
                .route("/watermarks/{id}/png", get(routes::watermarks::png))
                .layer(DefaultBodyLimit::max(32 * 1024 * 1024)),
        )
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            request_timeout,
        ))
        .layer(from_fn_with_state(state.clone(), count_timeouts))
        .merge(exports)
        .fallback(api_not_found)
        .layer(from_fn(auth_middleware))
        .layer(from_fn_with_state(state.clone(), inject_auth_context))
        .layer(from_fn_with_state(state.clone(), csrf_guard))
        .layer(from_fn_with_state(state.clone(), resolve_client_meta))
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
            .layer(
                CompressionLayer::new().compress_when(DefaultPredicate::new().and(
                    NotForContentType::const_new(routes::source::SOURCE_CONTENT_TYPE),
                )),
            )
            .layer(DefaultBodyLimit::max(body_bytes))
            .layer(RequestBodyLimitLayer::new(body_bytes))
            .layer(TimeoutLayer::with_status_code(
                StatusCode::REQUEST_TIMEOUT,
                request_timeout.max(export_timeout),
            ))
            .layer(cors),
    )
}
