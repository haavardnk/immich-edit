use std::path::Path;

use crate::common::*;
use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use axum::response::Response;
use immich_edit_backend::app;
use immich_edit_backend::routes::headers::{CACHE_IMMUTABLE, CACHE_REVALIDATE};
use tempfile::TempDir;
use tower::ServiceExt;
use wiremock::MockServer;

const SHELL: &str = "<!doctype html><title>shell</title>";
const INDEX: &str = "<!doctype html><title>index</title>";
const IMMUTABLE_CHUNK: &str = "/_app/immutable/chunks/app.B1x9.js";

fn web_dir(with_service_worker: bool) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let files = [
        ("200.html", SHELL),
        ("index.html", INDEX),
        ("manifest.webmanifest", "{\"name\":\"immich-edit\"}"),
        ("favicon.svg", "<svg xmlns=\"http://www.w3.org/2000/svg\"/>"),
        ("fonts/GoogleSans.ttf", "font"),
        ("licenses/OFL.txt", "licence"),
        ("_app/version.json", "{\"version\":\"1\"}"),
        ("_app/immutable/chunks/app.B1x9.js", "export {};"),
        ("_app/immutable/workers/assets/web_render_bg.wasm", "wasm"),
    ];
    for (name, body) in files {
        write(dir.path(), name, body);
    }
    if with_service_worker {
        write(dir.path(), "service-worker.js", "self.addEventListener;");
    }
    dir
}

fn write(root: &Path, name: &str, body: &str) {
    let path = root.join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, body).unwrap();
}

async fn web_app(server: &MockServer, dir: &TempDir) -> Router {
    let state = test_state(server).await;
    let token = seed_session(server, &state).await;
    wrap_auth(app::router_with_web_dir(state, dir.path()), token)
}

async fn signed_out_app(server: &MockServer, dir: &TempDir) -> Router {
    let state = test_state(server).await;
    state.instance.claim(&server.uri()).await.unwrap();
    let app = app::router_with_web_dir(state, dir.path());
    let api = fetch(&app, signed_out(Method::GET, "/api/health")).await;
    assert_eq!(api.status(), StatusCode::UNAUTHORIZED);
    app
}

fn signed_out(method: Method, path: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(path)
        .header("host", "localhost")
        .body(Body::empty())
        .unwrap()
}

async fn fetch(app: &Router, request: Request<Body>) -> Response {
    assert!(request.headers().get("cookie").is_none());
    app.clone().oneshot(request).await.unwrap()
}

fn assert_security_headers(resp: &Response, path: &str) {
    for (name, value) in [
        ("x-content-type-options", "nosniff"),
        ("referrer-policy", "no-referrer"),
        ("x-frame-options", "DENY"),
    ] {
        assert_eq!(
            header_str(resp, name).as_deref(),
            Some(value),
            "{name} on {path}"
        );
    }
}

fn assert_served(resp: &Response, path: &str, cache: &str) {
    assert_eq!(resp.status(), StatusCode::OK, "{path}");
    assert_eq!(
        header_str(resp, "cache-control").as_deref(),
        Some(cache),
        "{path}"
    );
    assert_security_headers(resp, path);
}

#[tokio::test]
async fn hashed_bundle_output_is_immutable() {
    let server = MockServer::start().await;
    let dir = web_dir(true);
    let app = signed_out_app(&server, &dir).await;
    for path in [
        IMMUTABLE_CHUNK,
        "/_app/immutable/workers/assets/web_render_bg.wasm",
    ] {
        let resp = fetch(&app, signed_out(Method::GET, path)).await;
        assert_served(&resp, path, CACHE_IMMUTABLE);
    }
}

#[tokio::test]
async fn update_sensitive_files_revalidate() {
    let server = MockServer::start().await;
    let dir = web_dir(true);
    let app = signed_out_app(&server, &dir).await;
    for path in [
        "/",
        "/service-worker.js",
        "/manifest.webmanifest",
        "/200.html",
        "/index.html",
        "/_app/version.json",
    ] {
        let resp = fetch(&app, signed_out(Method::GET, path)).await;
        assert_served(&resp, path, CACHE_REVALIDATE);
    }
}

#[tokio::test]
async fn other_static_files_revalidate() {
    let server = MockServer::start().await;
    let dir = web_dir(true);
    let app = signed_out_app(&server, &dir).await;
    for path in ["/favicon.svg", "/fonts/GoogleSans.ttf", "/licenses/OFL.txt"] {
        let resp = fetch(&app, signed_out(Method::GET, path)).await;
        assert_served(&resp, path, CACHE_REVALIDATE);
    }
}

#[tokio::test]
async fn client_routes_get_the_shell_and_revalidate() {
    let server = MockServer::start().await;
    let dir = web_dir(true);
    let app = signed_out_app(&server, &dir).await;
    for path in [
        "/photos",
        "/assets/123",
        "/settings/diagnostics",
        "/_app/immutable/",
    ] {
        let resp = fetch(&app, signed_out(Method::GET, path)).await;
        assert_served(&resp, path, CACHE_REVALIDATE);
        let content_type = header_str(&resp, "content-type").unwrap_or_default();
        assert!(
            content_type.starts_with("text/html"),
            "{path}: {content_type}"
        );
        assert_eq!(body_bytes(resp).await, SHELL.as_bytes(), "{path}");
    }
}

#[tokio::test]
async fn missing_files_are_not_found() {
    let server = MockServer::start().await;
    let dir = web_dir(false);
    let app = signed_out_app(&server, &dir).await;
    for path in [
        "/service-worker.js",
        "/missing.png",
        "/_app/immutable/chunks/gone.C2y0.js",
    ] {
        let resp = fetch(&app, signed_out(Method::GET, path)).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND, "{path}");
        assert_eq!(header_str(&resp, "cache-control"), None, "{path}");
        assert_security_headers(&resp, path);
        assert!(body_bytes(resp).await.is_empty(), "{path}");
    }
}

#[tokio::test]
async fn manifest_is_served_as_a_web_app_manifest() {
    let server = MockServer::start().await;
    let dir = web_dir(true);
    let app = signed_out_app(&server, &dir).await;
    let resp = fetch(&app, signed_out(Method::GET, "/manifest.webmanifest")).await;
    assert_eq!(
        header_str(&resp, "content-type").as_deref(),
        Some("application/manifest+json")
    );
}

#[tokio::test]
async fn revalidation_keeps_the_cache_policy() {
    let server = MockServer::start().await;
    let dir = web_dir(true);
    let app = signed_out_app(&server, &dir).await;
    for (path, cache) in [
        ("/index.html", CACHE_REVALIDATE),
        ("/photos", CACHE_REVALIDATE),
        (IMMUTABLE_CHUNK, CACHE_IMMUTABLE),
    ] {
        let first = fetch(&app, signed_out(Method::GET, path)).await;
        let etag = header_str(&first, "etag").expect("etag");
        let mut conditional = signed_out(Method::GET, path);
        conditional
            .headers_mut()
            .insert("if-none-match", etag.parse().unwrap());
        let resp = fetch(&app, conditional).await;
        assert_eq!(resp.status(), StatusCode::NOT_MODIFIED, "{path}");
        assert_eq!(
            header_str(&resp, "cache-control").as_deref(),
            Some(cache),
            "{path}"
        );
    }
}

#[tokio::test]
async fn head_requests_follow_the_same_rules() {
    let server = MockServer::start().await;
    let dir = web_dir(false);
    let app = signed_out_app(&server, &dir).await;
    let missing = fetch(&app, signed_out(Method::HEAD, "/service-worker.js")).await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    assert_eq!(header_str(&missing, "cache-control"), None);
    for (path, cache) in [
        ("/fonts/GoogleSans.ttf", CACHE_REVALIDATE),
        (IMMUTABLE_CHUNK, CACHE_IMMUTABLE),
        ("/photos", CACHE_REVALIDATE),
    ] {
        let get = fetch(&app, signed_out(Method::GET, path)).await;
        let head = fetch(&app, signed_out(Method::HEAD, path)).await;
        assert_served(&head, path, cache);
        assert_eq!(
            header_str(&head, "cache-control"),
            header_str(&get, "cache-control"),
            "{path}"
        );
        assert!(body_bytes(head).await.is_empty(), "{path}");
    }
}

#[tokio::test]
async fn api_paths_never_reach_the_web_dir() {
    let server = MockServer::start().await;
    let dir = web_dir(true);
    let app = web_app(&server, &dir).await;
    for path in ["/api/does/not/exist", "/api/missing.json"] {
        let resp = fetch(&app, get(path)).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND, "{path}");
        assert_eq!(header_str(&resp, "cache-control"), None, "{path}");
        assert_eq!(body_json(resp).await["code"], "not_found", "{path}");
    }
}
