use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::extract::{DefaultBodyLimit, Request, State};
use axum::http::header::{HeaderName, HeaderValue};
use axum::http::{Method, StatusCode};
use axum::middleware::{Next, from_fn, from_fn_with_state};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use tower::ServiceBuilder;
use tower_governor::GovernorLayer;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::GlobalKeyExtractor;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::request_id::{
    MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer,
};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use crate::error::{AppError, REQUEST_ID, api_not_found};
use crate::routes;
use crate::state::AppState;

const REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

#[derive(Clone, Default)]
struct UuidRequestId;

impl MakeRequestId for UuidRequestId {
    fn make_request_id<B>(&mut self, _req: &http::Request<B>) -> Option<RequestId> {
        let id = Uuid::new_v4().to_string();
        HeaderValue::from_str(&id).ok().map(RequestId::new)
    }
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

    let api = Router::new()
        .route("/health", get(routes::health::health))
        .route("/health/live", get(routes::health::live))
        .route("/auth/login", post(routes::auth::login))
        .route("/auth/logout", post(routes::auth::logout))
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
        .route("/search/statistics", post(routes::search::statistics))
        .route("/edits", get(routes::edits::list))
        .route(
            "/assets/{id}",
            get(routes::assets::detail).put(routes::assets::update),
        )
        .route("/assets/{id}/thumb", get(routes::assets::thumbnail))
        .route("/assets/{id}/edited-thumb", get(routes::edited_thumb::get))
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
        .route(
            "/assets/{id}/export",
            get(routes::export::get_export)
                .post(routes::export::post_export)
                .layer(heavy.clone()),
        )
        .route(
            "/assets/{id}/export/immich",
            post(routes::export::post_export_immich).layer(heavy),
        )
        .route("/rasters", post(routes::rasters::upload))
        .route("/rasters/{raster_id}", get(routes::rasters::get))
        .route("/rasters/{raster_id}/meta", get(routes::rasters::meta))
        .fallback(api_not_found)
        .layer(from_fn_with_state(state.clone(), debug_gate))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
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
                Duration::from_secs(60),
            ))
            .layer(cors),
    )
}

fn build_cors(allowed: &[String]) -> CorsLayer {
    let methods = [
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::OPTIONS,
    ];
    let base = CorsLayer::new()
        .allow_methods(methods)
        .allow_credentials(true)
        .allow_headers([
            HeaderName::from_static("content-type"),
            HeaderName::from_static("authorization"),
            HeaderName::from_static("x-request-id"),
        ]);
    if allowed.is_empty() {
        return base;
    }
    let origins: Vec<HeaderValue> = allowed
        .iter()
        .filter_map(|o| HeaderValue::from_str(o).ok())
        .collect();
    base.allow_origin(AllowOrigin::list(origins))
}

async fn auth_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let path = req.uri().path();
    if path == "/health/live" || path == "/auth/login" {
        return next.run(req).await;
    }
    let Some(expected) = state.config.auth_token.clone() else {
        return next.run(req).await;
    };
    let token = routes::auth::extract_token(req.headers());
    match token {
        Some(t) if routes::auth::ct_eq(t.as_bytes(), expected.as_bytes()) => next.run(req).await,
        _ => AppError::Unauthorized.into_response(),
    }
}

async fn debug_gate(State(state): State<AppState>, req: Request<Body>, next: Next) -> Response {
    if req.uri().path() == "/debug/timings" && !state.config.debug_endpoints {
        return AppError::NotFound.into_response();
    }
    next.run(req).await
}

async fn request_id_scope(req: Request<Body>, next: Next) -> Response {
    let id = req
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    REQUEST_ID.scope(id, next.run(req)).await
}
