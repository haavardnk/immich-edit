use std::path::Path;

use axum::Router;
use axum::extract::{Request, State};
use axum::http::header::CACHE_CONTROL;
use axum::http::{HeaderValue, StatusCode};
use axum::middleware::{Next, from_fn};
use axum::response::{IntoResponse, Response};
use tower::ServiceExt;
use tower_http::services::{ServeDir, ServeFile};

use crate::routes::headers::{CACHE_IMMUTABLE, CACHE_REVALIDATE};

const SHELL_FILE: &str = "200.html";
const IMMUTABLE_PREFIX: &str = "/_app/immutable/";

pub fn router(web_dir: &Path) -> Option<Router> {
    let shell_path = web_dir.join(SHELL_FILE);
    if !shell_path.exists() {
        return None;
    }
    let shell = Router::new()
        .fallback(shell)
        .with_state(ServeFile::new(shell_path));
    let files = ServeDir::new(web_dir).fallback(shell);
    Some(
        Router::new()
            .fallback_service(files)
            .layer(from_fn(apply_cache_policy)),
    )
}

async fn shell(State(file): State<ServeFile>, req: Request) -> Response {
    if names_a_file(req.uri().path()) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let mut resp = file.oneshot(req).await.into_response();
    resp.headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static(CACHE_REVALIDATE));
    resp
}

async fn apply_cache_policy(req: Request, next: Next) -> Response {
    let policy = cache_policy(req.uri().path());
    let mut resp = next.run(req).await;
    let status = resp.status();
    if status.is_success() || status == StatusCode::NOT_MODIFIED {
        resp.headers_mut()
            .entry(CACHE_CONTROL)
            .or_insert(HeaderValue::from_static(policy));
    }
    resp
}

fn cache_policy(path: &str) -> &'static str {
    if path.starts_with(IMMUTABLE_PREFIX) {
        CACHE_IMMUTABLE
    } else {
        CACHE_REVALIDATE
    }
}

fn names_a_file(path: &str) -> bool {
    let segment = path.rsplit('/').next().unwrap_or_default();
    segment
        .rsplit_once('.')
        .is_some_and(|(stem, extension)| !stem.is_empty() && !extension.is_empty())
}

#[cfg(test)]
mod tests;
