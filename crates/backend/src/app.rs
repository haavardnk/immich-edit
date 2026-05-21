use std::time::Duration;

use axum::Router;
use axum::http::StatusCode;
use axum::routing::{get, post};
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::services::{ServeDir, ServeFile};
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
        .route("/people", get(routes::people::list))
        .route("/people/{id}/thumb", get(routes::people::thumbnail))
        .route("/tags", get(routes::tags::list))
        .route("/folders/paths", get(routes::folders::paths))
        .route("/folders/assets", get(routes::folders::assets))
        .route("/search/metadata", post(routes::search::metadata))
        .route("/assets/{id}", get(routes::assets::detail))
        .route("/assets/{id}/thumb", get(routes::assets::thumbnail))
        .route(
            "/assets/{id}/edits",
            get(routes::edits::get)
                .put(routes::edits::put)
                .delete(routes::edits::delete),
        )
        .route("/assets/{id}/edits/auto", post(routes::edits::auto))
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

    let web_dir = std::env::var("WEB_DIR").unwrap_or_else(|_| "./web".into());
    let fallback_file = format!("{web_dir}/200.html");
    let has_web = std::path::Path::new(&fallback_file).exists();

    let mut root = Router::new().nest("/api", api);
    if has_web {
        let spa = ServeDir::new(&web_dir).fallback(ServeFile::new(&fallback_file));
        root = root.fallback_service(spa);
    }

    root.with_state(state).layer(
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
