use std::time::Duration;

use axum::body::{Body, HttpBody};
use axum::http::header::{CONTENT_TYPE, HeaderName};
use axum::http::{Request, Response, StatusCode};
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower_http::trace::{MakeSpan, OnResponse, TraceLayer};
use tracing::field::Empty;
use tracing::{Level, Span};

pub const REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

pub type HttpTraceLayer =
    TraceLayer<SharedClassifier<ServerErrorsAsFailures>, RequestSpan, (), AccessLog, (), (), ()>;

#[derive(Clone, Debug)]
pub struct ErrorReport {
    pub code: Option<&'static str>,
    pub message: String,
}

const REJECTION_BODY_LIMIT: u64 = 2048;

#[derive(Clone, Copy, Debug, Default)]
pub struct RequestSpan;

impl<B> MakeSpan<B> for RequestSpan {
    fn make_span(&mut self, request: &Request<B>) -> Span {
        let request_id = request
            .headers()
            .get(REQUEST_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        tracing::error_span!(
            target: "http",
            "request",
            request_id,
            method = %request.method(),
            path = request.uri().path(),
            owner = Empty,
            session = Empty,
        )
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct AccessLog;

impl<B> OnResponse<B> for AccessLog {
    fn on_response(self, response: &Response<B>, latency: Duration, _span: &Span) {
        let status = response.status().as_u16();
        let latency_ms = u64::try_from(latency.as_millis()).unwrap_or(u64::MAX);
        let report = response.extensions().get::<ErrorReport>();
        let code = report.and_then(|r| r.code);
        let detail = report.map(|r| r.message.as_str());
        match access_level(response.status()) {
            Level::ERROR => {
                tracing::error!(target: "http", status, latency_ms, code, detail, "request failed")
            }
            Level::WARN => {
                tracing::warn!(target: "http", status, latency_ms, code, detail, "request rejected")
            }
            Level::INFO => {
                tracing::info!(target: "http", status, latency_ms, code, detail, "request refused")
            }
            _ => {
                tracing::debug!(target: "http", status, latency_ms, code, detail, "request served")
            }
        }
    }
}

pub fn trace_layer() -> HttpTraceLayer {
    TraceLayer::new_for_http()
        .make_span_with(RequestSpan)
        .on_request(())
        .on_response(AccessLog)
        .on_body_chunk(())
        .on_eos(())
        .on_failure(())
}

pub async fn report_rejection(response: Response<Body>) -> Response<Body> {
    if !response.status().is_client_error() || response.extensions().get::<ErrorReport>().is_some()
    {
        return response;
    }
    let is_text = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("text/plain"));
    let len = response.body().size_hint().exact().unwrap_or(u64::MAX);
    if !is_text || len == 0 || len > REJECTION_BODY_LIMIT {
        return response;
    }
    let (mut parts, body) = response.into_parts();
    let Ok(bytes) = axum::body::to_bytes(body, REJECTION_BODY_LIMIT as usize).await else {
        return Response::from_parts(parts, Body::empty());
    };
    parts.extensions.insert(ErrorReport {
        code: None,
        message: String::from_utf8_lossy(&bytes).into_owned(),
    });
    Response::from_parts(parts, Body::from(bytes))
}

pub fn access_level(status: StatusCode) -> Level {
    match status.as_u16() {
        500.. => Level::ERROR,
        400 | 408 | 409 | 413 | 422 | 429 => Level::WARN,
        401 | 404 => Level::DEBUG,
        402..=499 => Level::INFO,
        _ => Level::DEBUG,
    }
}

pub fn record_identity(owner: uuid::Uuid, session: uuid::Uuid) {
    let span = Span::current();
    span.record("owner", tracing::field::display(owner));
    span.record("session", tracing::field::display(session));
}

#[cfg(test)]
mod tests {
    use super::access_level;
    use axum::http::StatusCode;
    use tracing::Level;

    #[test]
    fn maps_status_to_level() {
        let cases = [
            (200, Level::DEBUG),
            (304, Level::DEBUG),
            (400, Level::WARN),
            (401, Level::DEBUG),
            (403, Level::INFO),
            (404, Level::DEBUG),
            (405, Level::INFO),
            (408, Level::WARN),
            (409, Level::WARN),
            (413, Level::WARN),
            (422, Level::WARN),
            (428, Level::INFO),
            (429, Level::WARN),
            (500, Level::ERROR),
            (502, Level::ERROR),
            (504, Level::ERROR),
        ];
        for (code, want) in cases {
            let status = StatusCode::from_u16(code).unwrap();
            let got = access_level(status);
            if got != want {
                panic!("{code}: got {got}, want {want}");
            }
        }
    }
}
