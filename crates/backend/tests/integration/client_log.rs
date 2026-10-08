use crate::common::*;
use axum::http::StatusCode;
use serde_json::{Value, json};
use tower::ServiceExt;
use tracing::Level;
use wiremock::MockServer;

fn report(message: &str) -> Value {
    json!({
        "kind": "render_worker",
        "level": "error",
        "message": message,
        "stack": "Error: boom\n    at render (worker.ts:1:1)",
        "route": "/edit/abc"
    })
}

#[tokio::test]
async fn a_browser_error_is_logged_as_one_escaped_line() {
    let (capture, _guard) = capture_logs(Level::WARN);
    let server = MockServer::start().await;
    let app = test_app(&server).await;

    let resp = app
        .oneshot(json_request(
            "POST",
            "/api/client-log",
            report("line one\n\u{1b}[31mforged line"),
        ))
        .await
        .unwrap();

    if resp.status() != StatusCode::NO_CONTENT {
        panic!("status {}", resp.status());
    }
    let lines = capture.lines_with("browser error");
    let [line] = lines.as_slice() else {
        panic!("expected one line, got {:?}", capture.text());
    };
    for needle in [
        "kind=\"render_worker\"",
        "route=\"/edit/abc\"",
        "line one\\n",
        "at render (worker.ts:1:1)",
    ] {
        if !line.contains(needle) {
            panic!("line misses {needle:?}: {line}");
        }
    }
    let text = capture.text();
    if text.contains('\u{1b}') || text.lines().any(|l| l.starts_with("forged line")) {
        panic!("browser text must be escaped: {text:?}");
    }
}

#[tokio::test]
async fn browser_errors_over_the_rate_limit_are_dropped_with_one_warning() {
    let (capture, _guard) = capture_logs(Level::WARN);
    let server = MockServer::start().await;
    let app = test_app(&server).await;

    for i in 0..35 {
        let resp = app
            .clone()
            .oneshot(json_request(
                "POST",
                "/api/client-log",
                report(&format!("error {i}")),
            ))
            .await
            .unwrap();
        if resp.status() != StatusCode::NO_CONTENT {
            panic!("report {i}: status {}", resp.status());
        }
    }

    let logged = capture.lines_with("browser error").len();
    let warned = capture.lines_with("rate limit reached").len();
    if logged != 30 || warned != 1 {
        panic!("logged {logged}, warned {warned}");
    }
}

#[tokio::test]
async fn client_log_refuses_anonymous_and_oversized_reports() {
    let server = MockServer::start().await;
    let anonymous = router(test_state(&server).await);
    let signed_in = test_app(&server).await;
    let oversized = report(&"x".repeat(20 * 1024));
    let cases = [
        (anonymous, report("hi"), StatusCode::UNAUTHORIZED),
        (signed_in, oversized, StatusCode::PAYLOAD_TOO_LARGE),
    ];
    for (app, body, want) in cases {
        let resp = app
            .oneshot(json_request("POST", "/api/client-log", body))
            .await
            .unwrap();
        if resp.status() != want {
            panic!("want {want}, got {}", resp.status());
        }
    }
}
