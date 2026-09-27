use std::sync::Arc;

use serde::Serialize;
use sqlx::SqlitePool;
use uuid::Uuid;
use uuid::fmt::Hyphenated;

use crate::immich::client::{ImmichAuth, ImmichUser};
use crate::services::crypto::{InstanceCrypto, SecretBytes};

mod claims;
mod sessions;
#[cfg(test)]
mod tests;
mod users;

#[derive(Debug, thiserror::Error)]
pub enum AuthStoreError {
    #[error("db: {0}")]
    Db(#[from] sqlx::Error),
    #[error("crypto: {0}")]
    Crypto(#[from] crate::services::crypto::CryptoError),
    #[error("already configured")]
    AlreadyConfigured,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(rename_all = "lowercase")]
pub enum AuthKind {
    Password,
    ApiKey,
    OAuth,
}

impl AuthKind {
    pub fn immich_auth(self, cred: String) -> ImmichAuth {
        match self {
            Self::Password | Self::OAuth => ImmichAuth::Bearer(cred),
            Self::ApiKey => ImmichAuth::ApiKey(cred),
        }
    }

    pub fn revokes_upstream(self) -> bool {
        matches!(self, Self::Password | Self::OAuth)
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserRecord {
    #[sqlx(try_from = "Hyphenated")]
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub is_admin: bool,
    pub access_enabled: bool,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SessionRecord {
    #[sqlx(try_from = "Hyphenated")]
    pub id: Uuid,
    #[sqlx(try_from = "Hyphenated")]
    pub user_id: Uuid,
    pub auth_kind: AuthKind,
    pub server_epoch: i64,
    pub created_at: String,
    pub expires_at: String,
    pub last_seen_at: String,
    pub user_agent: Option<String>,
    pub ip: Option<String>,
}

pub struct AuthContext {
    pub user: UserRecord,
    pub session_id: Uuid,
    pub server_epoch: i64,
    pub auth_kind: AuthKind,
    pub immich_cred: SecretBytes,
}

#[derive(Clone)]
pub struct AuthStore {
    pool: SqlitePool,
    crypto: Arc<InstanceCrypto>,
}

impl AuthStore {
    pub fn new(pool: SqlitePool, crypto: Arc<InstanceCrypto>) -> Self {
        Self { pool, crypto }
    }
}
