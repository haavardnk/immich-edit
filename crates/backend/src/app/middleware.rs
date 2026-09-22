use super::REQUEST_ID_HEADER;
use crate::config::Cidr;
use crate::error::{AppError, REQUEST_ID};
use crate::routes;
use crate::state::AppState;
use axum::body::Body;
use axum::extract::{ConnectInfo, Request, State};
use axum::http::header::{COOKIE, HOST, ORIGIN};
use axum::http::{Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

pub async fn inject_auth_context(
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

pub async fn auth_middleware(req: Request<Body>, next: Next) -> Response {
    let path = req.uri().path();
    if matches!(
        path,
        "/health/live"
            | "/auth/logout"
            | "/auth/login/password"
            | "/auth/login/api-key"
            | "/auth/providers"
            | "/auth/oauth/start"
            | "/auth/oauth/callback"
            | "/setup/status"
            | "/setup/complete"
            | "/setup/providers"
            | "/setup/oauth/start"
            | "/setup/oauth/complete"
    ) {
        return next.run(req).await;
    }
    if req.extensions().get::<routes::auth::AuthCtx>().is_some() {
        return next.run(req).await;
    }
    AppError::Unauthorized.into_response()
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

pub async fn resolve_client_meta(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let peer = req
        .extensions()
        .get::<ConnectInfo<std::net::SocketAddr>>()
        .map(|c| c.0.ip());
    let trust_forwarded = peer.is_some_and(|ip| is_trusted_peer(ip, &state.config.trusted_proxies));
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

fn is_trusted_peer(ip: std::net::IpAddr, trusted: &[Cidr]) -> bool {
    trusted.iter().any(|c| c.contains(ip))
}

pub async fn csrf_guard(State(state): State<AppState>, req: Request<Body>, next: Next) -> Response {
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

pub async fn request_id_scope(req: Request<Body>, next: Next) -> Response {
    let id = req
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    REQUEST_ID.scope(id, next.run(req)).await
}

pub async fn count_timeouts(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let res = next.run(req).await;
    if res.status() == StatusCode::REQUEST_TIMEOUT {
        state.render.telemetry().record_request_timeout();
    }
    res
}

#[cfg(test)]
mod tests {
    use super::is_trusted_peer;
    use crate::config::Cidr;

    fn parse(list: &[&str]) -> Vec<Cidr> {
        list.iter().map(|raw| raw.parse().expect(raw)).collect()
    }

    #[test]
    fn trusts_only_local_peers_by_default() {
        let trusted = crate::config::default_trusted_proxies();
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
            if is_trusted_peer(ip, &trusted) != expected {
                panic!("{raw} should be trusted={expected}");
            }
        }
    }

    #[test]
    fn trusts_a_configured_public_proxy() {
        let trusted = parse(&["203.0.113.7"]);
        if !is_trusted_peer("203.0.113.7".parse().unwrap(), &trusted) {
            panic!("configured proxy must be trusted");
        }
        if is_trusted_peer("10.0.0.1".parse().unwrap(), &trusted) {
            panic!("an explicit list replaces the private defaults");
        }
    }

    #[test]
    fn an_empty_list_trusts_nobody() {
        if is_trusted_peer("127.0.0.1".parse().unwrap(), &[]) {
            panic!("empty list must not trust loopback");
        }
    }
}
