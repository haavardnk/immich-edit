use axum::extract::FromRequestParts;
use axum::http::header::{AUTHORIZATION, HeaderMap};
use axum::http::request::Parts;
use axum_extra::extract::cookie::CookieJar;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

use crate::error::AppError;
use crate::immich::client::ImmichClient;
use crate::immich::url::resolve_immich_base;
use crate::services::auth_store::{AuthContext, AuthKind};
use crate::services::crypto::SecretBytes;
use crate::services::login_session::{AUTH_COOKIE, ClientMeta};
use crate::state::AppState;

#[derive(Clone)]
pub struct AuthCtx {
    pub owner: Uuid,
    pub session_id: Uuid,
    pub server_epoch: i64,
    pub is_admin: bool,
    pub immich: ImmichClient,
    pub cred: Arc<SecretBytes>,
    pub auth_kind: AuthKind,
}

impl From<&AuthCtx> for crate::services::render::RenderIdentity {
    fn from(ctx: &AuthCtx) -> Self {
        Self {
            owner: ctx.owner,
            server_epoch: ctx.server_epoch,
        }
    }
}

impl<S: Send + Sync> FromRequestParts<S> for AuthCtx {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, AppError> {
        parts
            .extensions
            .get::<AuthCtx>()
            .cloned()
            .ok_or(AppError::Unauthorized)
    }
}

pub struct AdminCtx(pub AuthCtx);

impl<S: Send + Sync> FromRequestParts<S> for AdminCtx {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, AppError> {
        let ctx = AuthCtx::from_request_parts(parts, state).await?;
        if ctx.is_admin {
            Ok(AdminCtx(ctx))
        } else {
            Err(AppError::AdminRequired)
        }
    }
}

impl<S: Send + Sync> FromRequestParts<S> for ClientMeta {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(parts
            .extensions
            .get::<ClientMeta>()
            .cloned()
            .unwrap_or_default())
    }
}

pub async fn build_auth_ctx(state: &AppState, headers: &HeaderMap) -> Option<AuthCtx> {
    let token = extract_token(headers)?;
    let actx = state.auth.authenticate(&token).await.ok()??;
    let base = resolve_immich_base(&state.instance).await.ok()?;
    let cred = actx.immich_cred.to_utf8()?;
    let auth = actx.auth_kind.immich_auth(cred);
    let immich = ImmichClient::with_auth(
        base,
        auth,
        Duration::from_secs(state.config.original_timeout_secs),
    )
    .ok()?;
    Some(AuthCtx {
        owner: actx.user.id,
        session_id: actx.session_id,
        server_epoch: actx.server_epoch,
        is_admin: actx.user.is_admin,
        immich,
        cred: Arc::new(actx.immich_cred),
        auth_kind: actx.auth_kind,
    })
}

pub fn extract_token(headers: &HeaderMap) -> Option<String> {
    if let Some(auth) = headers.get(AUTHORIZATION).and_then(|v| v.to_str().ok()) {
        if let Some(rest) = auth.strip_prefix("Bearer ") {
            return Some(rest.to_string());
        }
    }
    CookieJar::from_headers(headers)
        .get(AUTH_COOKIE)
        .map(|c| c.value().to_string())
}

pub async fn require_session(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AuthContext, AppError> {
    let token = extract_token(headers).ok_or(AppError::Unauthorized)?;
    match state.auth.authenticate(&token).await {
        Ok(Some(ctx)) => Ok(ctx),
        Ok(None) => Err(AppError::Unauthorized),
        Err(_) => Err(AppError::Internal),
    }
}
