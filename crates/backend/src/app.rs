use std::time::Duration;

use axum::Router;
use axum::http::StatusCode;
use axum::routing::get;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::error::api_not_found;
use crate::routes;
use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/health", get(routes::health::health))
        .route("/albums", get(routes::albums::list))
        .route("/albums/{id}", get(routes::albums::detail))
        .route("/assets/{id}", get(routes::assets::detail))
        .route("/assets/{id}/thumb", get(routes::assets::thumbnail))
        .route(
            "/assets/{id}/edits",
            get(routes::edits::get)
                .put(routes::edits::put)
                .delete(routes::edits::delete),
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
            get(routes::export::get_export).post(routes::export::post_export),
        )
        .fallback(api_not_found);

    Router::new()
        .nest("/api", api)
        .fallback(spa_fallback)
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(RequestBodyLimitLayer::new(16 * 1024 * 1024))
                .layer(TimeoutLayer::with_status_code(
                    StatusCode::REQUEST_TIMEOUT,
                    Duration::from_secs(60),
                ))
                .layer(CorsLayer::permissive()),
        )
}

async fn spa_fallback() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "not found")
}
