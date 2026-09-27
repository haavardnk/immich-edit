use std::time::Duration;

use url::Url;

use crate::error::AppError;
use crate::immich::ImmichError;
use crate::immich::client::{ImmichAuth, ImmichClient, ImmichUser};
use crate::services::auth_store::AuthKind;

const VALIDATE_TIMEOUT: Duration = Duration::from_secs(30);

pub async fn validate_password(
    base: &Url,
    email: &str,
    password: &str,
) -> Result<(ImmichUser, Vec<u8>), AppError> {
    let candidate = ImmichClient::with_auth(
        base.clone(),
        ImmichAuth::ApiKey(String::new()),
        VALIDATE_TIMEOUT,
    )?;
    let login = candidate
        .login_password(email, password)
        .await
        .map_err(map_login_error)?;
    let user = ImmichUser {
        id: login.user_id,
        email: login.user_email,
        name: login.name,
        is_admin: login.is_admin,
    };
    Ok((user, login.access_token.into_bytes()))
}

pub async fn validate_api_key(base: &Url, api_key: &str) -> Result<ImmichUser, AppError> {
    let client = ImmichClient::with_auth(
        base.clone(),
        ImmichAuth::ApiKey(api_key.to_string()),
        VALIDATE_TIMEOUT,
    )?;
    client.me().await.map_err(map_login_error)
}

pub async fn validate_credentials(
    base: &Url,
    email: Option<&str>,
    password: Option<&str>,
    api_key: Option<&str>,
) -> Result<(ImmichUser, AuthKind, Vec<u8>), AppError> {
    if let Some(api_key) = api_key {
        let user = validate_api_key(base, api_key).await?;
        return Ok((user, AuthKind::ApiKey, api_key.as_bytes().to_vec()));
    }
    if let (Some(email), Some(password)) = (email, password) {
        let (user, cred) = validate_password(base, email, password).await?;
        return Ok((user, AuthKind::Password, cred));
    }
    Err(AppError::BadRequest(
        "email+password or api_key required".into(),
    ))
}

fn map_login_error(err: ImmichError) -> AppError {
    match err {
        ImmichError::Unauthorized => AppError::Unauthorized,
        other => other.into(),
    }
}
