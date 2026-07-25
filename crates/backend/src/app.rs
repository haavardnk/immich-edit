use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::extract::{ConnectInfo, DefaultBodyLimit, Request, State};
use axum::http::header::{COOKIE, HOST, HeaderName, HeaderValue, ORIGIN};
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

#[cfg(feature = "segment")]
fn model_routes() -> Router<AppState> {
    Router::new()
        .route("/masks/models", get(routes::models::list))
        .route("/assets/{id}/masks/generate", post(routes::masks::generate))
        .route("/masks/rebake", post(routes::masks::rebake))
        .route("/admin/masks/default", put(routes::models::select))
        .route(
            "/admin/models/{id}",
            post(routes::models::install).delete(routes::models::remove),
        )
}

#[cfg(not(feature = "segment"))]
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
        .fallback(api_not_found)
        .layer(from_fn_with_state(state.clone(), debug_gate))
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

async fn inject_auth_context(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let headers = req.headers().clone();
    if let Some(ctx) = routes::auth::build_auth_ctx(&state, &headers).await {
        req.extensions_mut().insert(ctx);
    }
    next.run(req).await
}

async fn auth_middleware(req: Request<Body>, next: Next) -> Response {
    let path = req.uri().path();
    if matches!(
        path,
        "/health/live"
            | "/auth/logout"
            | "/auth/login/password"
            | "/auth/login/api-key"
            | "/setup/status"
            | "/setup/complete"
    ) {
        return next.run(req).await;
    }
    if req.extensions().get::<routes::auth::AuthCtx>().is_some() {
        return next.run(req).await;
    }
    AppError::Unauthorized.into_response()
}

async fn debug_gate(State(state): State<AppState>, req: Request<Body>, next: Next) -> Response {
    if req.uri().path() == "/debug/timings" && !state.config.debug_endpoints {
        return AppError::NotFound.into_response();
    }
    next.run(req).await
}

fn origin_allowed(origin: &str, host: Option<&str>, allowed: &[String]) -> bool {
    if allowed.iter().any(|a| a == origin) {
        return true;
    }
    let Ok(url) = url::Url::parse(origin) else {
        return false;
    };
    let Some(origin_host) = url.host_str() else {
        return false;
    };
    let Some(host) = host else {
        return false;
    };
    let with_port = match url.port() {
        Some(p) => format!("{origin_host}:{p}"),
        None => origin_host.to_string(),
    };
    host == with_port || host == origin_host
}

async fn resolve_client_meta(mut req: Request<Body>, next: Next) -> Response {
    let peer = req
        .extensions()
        .get::<ConnectInfo<std::net::SocketAddr>>()
        .map(|c| c.0.ip());
    let trust_forwarded = peer.is_some_and(is_trusted_peer);
    let (ip, secure) = if trust_forwarded {
        let forwarded = req
            .headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(',').next())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let secure = req
            .headers()
            .get("x-forwarded-proto")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.eq_ignore_ascii_case("https"))
            .unwrap_or(false);
        (forwarded.or_else(|| peer.map(|p| p.to_string())), secure)
    } else {
        (peer.map(|p| p.to_string()), false)
    };
    req.extensions_mut().insert(routes::auth::ClientMeta {
        ip: ip.unwrap_or_else(|| "unknown".into()),
        secure,
    });
    next.run(req).await
}

fn is_trusted_peer(ip: std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => v4.is_loopback() || v4.is_private() || v4.is_link_local(),
        std::net::IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unique_local()
                || v6.is_unicast_link_local()
                || v6
                    .to_ipv4_mapped()
                    .is_some_and(|m| m.is_loopback() || m.is_private() || m.is_link_local())
        }
    }
}

async fn csrf_guard(State(state): State<AppState>, req: Request<Body>, next: Next) -> Response {
    if matches!(
        *req.method(),
        Method::GET | Method::HEAD | Method::OPTIONS | Method::TRACE
    ) {
        return next.run(req).await;
    }
    let headers = req.headers();
    let Some(origin) = headers.get(ORIGIN).and_then(|v| v.to_str().ok()) else {
        if headers.get(COOKIE).is_some() {
            return AppError::Forbidden.into_response();
        }
        return next.run(req).await;
    };
    let host = headers.get(HOST).and_then(|v| v.to_str().ok());
    if origin_allowed(origin, host, &state.config.allowed_origins) {
        return next.run(req).await;
    }
    AppError::Forbidden.into_response()
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

#[cfg(test)]
mod tests {
    use super::is_trusted_peer;

    #[test]
    fn trusts_only_local_peers() {
        let cases = [
            ("127.0.0.1", true),
            ("10.1.2.3", true),
            ("192.168.1.10", true),
            ("172.16.0.5", true),
            ("169.254.1.1", true),
            ("::1", true),
            ("fd00::1", true),
            ("fe80::1", true),
            ("::ffff:127.0.0.1", true),
            ("::ffff:10.0.0.1", true),
            ("8.8.8.8", false),
            ("172.32.0.1", false),
            ("2001:4860:4860::8888", false),
            ("::ffff:8.8.8.8", false),
        ];
        for (raw, expected) in cases {
            let ip = raw.parse().unwrap();
            assert_eq!(is_trusted_peer(ip), expected, "{raw}");
        }
    }
}
