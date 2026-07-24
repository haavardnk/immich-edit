use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::error::AppError;
use crate::routes::auth::{self, AdminCtx};
use crate::services::auth_store::AuthKind;
use crate::state::AppState;

pub async fn list_users(
    State(state): State<AppState>,
    _admin: AdminCtx,
) -> Result<Response, AppError> {
    let users = state
        .auth
        .list_users()
        .await
        .map_err(|_| AppError::Internal)?;
    let body: Vec<serde_json::Value> = users
        .iter()
        .map(|u| {
            json!({
                "id": u.id,
                "email": u.email,
                "name": u.name,
                "is_admin": u.is_admin,
                "access_enabled": u.access_enabled,
            })
        })
        .collect();
    Ok((StatusCode::OK, Json(json!({ "users": body }))).into_response())
}

#[derive(Deserialize)]
pub struct AccessBody {
    pub enabled: bool,
}

pub async fn set_access(
    State(state): State<AppState>,
    admin: AdminCtx,
    Path(id): Path<Uuid>,
    Json(body): Json<AccessBody>,
) -> Result<Response, AppError> {
    if id == admin.0.owner && !body.enabled {
        return Err(AppError::BadRequest(
            "cannot disable your own account".into(),
        ));
    }
    state
        .auth
        .get_user(id)
        .await
        .map_err(|_| AppError::Internal)?
        .ok_or(AppError::NotFound)?;
    state
        .auth
        .set_access(id, body.enabled)
        .await
        .map_err(|_| AppError::Internal)?;
    if !body.enabled {
        state
            .auth
            .revoke_all_for_user(id)
            .await
            .map_err(|_| AppError::Internal)?;
        state
            .jobs
            .cancel_active_for_owner(id)
            .await
            .map_err(|_| AppError::Internal)?;
    }
    Ok((StatusCode::OK, Json(json!({ "ok": true }))).into_response())
}

pub async fn purge_user_data(
    State(state): State<AppState>,
    _admin: AdminCtx,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    state
        .auth
        .get_user(id)
        .await
        .map_err(|_| AppError::Internal)?
        .ok_or(AppError::NotFound)?;
    state
        .jobs
        .purge_owner(id)
        .await
        .map_err(|_| AppError::Internal)?;
    state
        .edits
        .purge_owner(id)
        .await
        .map_err(|_| AppError::Internal)?;
    state
        .auth
        .revoke_all_for_user(id)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok((StatusCode::OK, Json(json!({ "ok": true }))).into_response())
}

pub async fn instance_info(
    State(state): State<AppState>,
    _admin: AdminCtx,
) -> Result<Response, AppError> {
    let cfg = state.instance.get().await.map_err(|_| AppError::Internal)?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "server_epoch": cfg.server_epoch,
            "immich_url": cfg.immich_url,
            "configured_at": cfg.configured_at,
        })),
    )
        .into_response())
}

#[derive(Deserialize)]
pub struct RebindBody {
    pub immich_url: String,
    pub confirm_hostname: String,
    pub email: Option<String>,
    pub password: Option<String>,
    pub api_key: Option<String>,
}

pub async fn rebind(
    State(state): State<AppState>,
    _admin: AdminCtx,
    headers: HeaderMap,
    Json(body): Json<RebindBody>,
) -> Result<Response, AppError> {
    let base = auth::validate_candidate_url(&body.immich_url)?;
    let host = base.host_str().unwrap_or_default();
    if !host.eq_ignore_ascii_case(body.confirm_hostname.trim()) {
        return Err(AppError::BadRequest(
            "confirmation hostname does not match".into(),
        ));
    }

    let (user, kind, cred): (_, _, Vec<u8>) = if let Some(api_key) = body.api_key.as_deref() {
        let user = auth::validate_api_key(&base, api_key).await?;
        (user, AuthKind::ApiKey, api_key.as_bytes().to_vec())
    } else if let (Some(email), Some(password)) = (body.email.as_deref(), body.password.as_deref())
    {
        let (user, cred) = auth::validate_password(&base, email, password).await?;
        (user, AuthKind::Password, cred)
    } else {
        return Err(AppError::BadRequest(
            "email+password or api_key required".into(),
        ));
    };

    if !user.is_admin {
        return Err(AppError::AdminRequired);
    }

    state
        .instance
        .rebind(base.as_str())
        .await
        .map_err(|_| AppError::Internal)?;
    state
        .jobs
        .purge_all()
        .await
        .map_err(|_| AppError::Internal)?;
    state
        .edits
        .purge_all()
        .await
        .map_err(|_| AppError::Internal)?;
    state
        .auth
        .purge_all()
        .await
        .map_err(|_| AppError::Internal)?;

    let ip = auth::client_ip(&headers);
    auth::finish_login(&state, &user, kind, &cred, &headers, Some(ip.as_str())).await
}
