mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::*;
use std::io::Write;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;
use tracing::Level;
use tracing_subscriber::fmt::MakeWriter;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<u8>>>);

impl Capture {
    fn text(&self) -> String {
        String::from_utf8_lossy(&self.0.lock().unwrap()).into_owned()
    }
}

impl Write for Capture {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for Capture {
    type Writer = Self;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

#[tokio::test]
async fn a_failed_upstream_logout_is_logged() {
    let capture = Capture::default();
    let subscriber = tracing_subscriber::fmt()
        .with_writer(capture.clone())
        .with_max_level(Level::WARN)
        .with_ansi(false)
        .finish();
    let _guard = tracing::subscriber::set_default(subscriber);

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/auth/logout"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    let app = password_app(&server).await;

    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/logout")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    if res.status() != StatusCode::OK {
        panic!(
            "logout must still clear the local session, got {}",
            res.status()
        );
    }
    let logged = capture.text();
    if !logged.contains("upstream logout failed") {
        panic!("a failed upstream logout must be logged, got {logged:?}");
    }
}
