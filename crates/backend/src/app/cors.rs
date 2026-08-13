use axum::http::Method;
use axum::http::header::{HeaderName, HeaderValue};
use tower_http::cors::{AllowOrigin, CorsLayer};

pub fn build_cors(allowed: &[String]) -> CorsLayer {
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
