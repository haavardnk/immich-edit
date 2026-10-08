use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Deserialize;

use crate::routes::auth::AuthCtx;
use crate::services::client_log_limiter::Admission;
use crate::state::AppState;

pub const BODY_LIMIT: usize = 16 * 1024;
const MAX_MESSAGE: usize = 2000;
const MAX_STACK: usize = 8000;
const MAX_ROUTE: usize = 512;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientLogKind {
    Exception,
    UnhandledRejection,
    Svelte,
    RenderWorker,
    Gpu,
}

impl ClientLogKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Exception => "exception",
            Self::UnhandledRejection => "unhandled_rejection",
            Self::Svelte => "svelte",
            Self::RenderWorker => "render_worker",
            Self::Gpu => "gpu",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientLogLevel {
    Error,
    Warn,
}

#[derive(Debug, Deserialize)]
pub struct ClientLogBody {
    pub kind: ClientLogKind,
    pub level: ClientLogLevel,
    pub message: String,
    #[serde(default)]
    pub stack: Option<String>,
    pub route: String,
}

fn clip(text: &str, max_chars: usize) -> &str {
    match text.char_indices().nth(max_chars) {
        Some((end, _)) => &text[..end],
        None => text,
    }
}

pub async fn post(
    State(state): State<AppState>,
    ctx: AuthCtx,
    Json(body): Json<ClientLogBody>,
) -> StatusCode {
    match state.client_log_limiter.admit(ctx.owner) {
        Admission::Accept => {}
        Admission::FirstDrop => {
            tracing::warn!(target: "client", "client log rate limit reached; dropping reports until the minute ends");
            return StatusCode::NO_CONTENT;
        }
        Admission::Drop => return StatusCode::NO_CONTENT,
    }
    let kind = body.kind.as_str();
    let error = clip(&body.message, MAX_MESSAGE);
    let stack = body.stack.as_deref().map(|s| clip(s, MAX_STACK));
    let route = clip(&body.route, MAX_ROUTE);
    match body.level {
        ClientLogLevel::Error => {
            tracing::error!(target: "client", kind, error, stack, route, "browser error")
        }
        ClientLogLevel::Warn => {
            tracing::warn!(target: "client", kind, error, stack, route, "browser warning")
        }
    }
    StatusCode::NO_CONTENT
}

#[cfg(test)]
mod tests {
    use super::clip;

    #[test]
    fn clips_on_char_boundaries() {
        let cases = [
            ("short", 10, "short"),
            ("exact", 5, "exact"),
            ("truncate me", 8, "truncate"),
            ("æøå-ok", 3, "æøå"),
            ("", 4, ""),
        ];
        for (text, max, want) in cases {
            let got = clip(text, max);
            if got != want {
                panic!("{text:?}/{max}: got {got:?}, want {want:?}");
            }
        }
    }
}
