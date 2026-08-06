use std::sync::Arc;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{Duration, Utc};
use rand::Rng;
use sha2::{Digest, Sha256};
use sqlx::{Row, Sqlite, SqlitePool, Transaction};
use uuid::Uuid;

use crate::immich::client::ImmichUser;
use crate::services::crypto::{Encrypted, InstanceCrypto, SecretBytes};

const IDLE_DAYS: i64 = 30;
const ABSOLUTE_DAYS: i64 = 90;
const LAST_SEEN_THROTTLE_SECS: i64 = 300;

#[derive(Debug, thiserror::Error)]
pub enum AuthStoreError {
    #[error("db: {0}")]
    Db(#[from] sqlx::Error),
    #[error("crypto: {0}")]
    Crypto(#[from] crate::services::crypto::CryptoError),
    #[error("already configured")]
    AlreadyConfigured,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthKind {
    Password,
    ApiKey,
}

impl AuthKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Password => "password",
            Self::ApiKey => "apikey",
        }
    }

    pub fn from_wire(s: &str) -> Self {
        match s {
            "apikey" => Self::ApiKey,
            _ => Self::Password,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserRecord {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub is_admin: bool,
    pub access_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct SessionRecord {
    pub id: Uuid,
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

struct PreparedSession {
    token: String,
    token_hash: Vec<u8>,
    encrypted: Encrypted,
    created_at: String,
    expires_at: String,
    absolute_expires_at: String,
}

fn hash_token(token: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hasher.finalize().to_vec()
}

impl AuthStore {
    pub fn new(pool: SqlitePool, crypto: Arc<InstanceCrypto>) -> Self {
        Self { pool, crypto }
    }

    pub async fn upsert_user(&self, user: &ImmichUser) -> Result<UserRecord, AuthStoreError> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO users (id, email, name, is_admin, access_enabled, created_at, last_login_at) \
             VALUES (?1, ?2, ?3, ?4, 1, ?5, ?5) \
             ON CONFLICT(id) DO UPDATE SET \
               email = excluded.email, \
               name = excluded.name, \
               is_admin = excluded.is_admin, \
               last_login_at = excluded.last_login_at",
        )
        .bind(user.id.to_string())
        .bind(&user.email)
        .bind(&user.name)
        .bind(user.is_admin as i64)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        self.get_user(user.id)
            .await?
            .ok_or(AuthStoreError::Db(sqlx::Error::RowNotFound))
    }

    pub async fn get_user(&self, id: Uuid) -> Result<Option<UserRecord>, AuthStoreError> {
        let row = sqlx::query(
            "SELECT id, email, name, is_admin, access_enabled FROM users WHERE id = ?1",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;
        row.as_ref().map(user_from_row).transpose()
    }

    pub async fn list_users(&self) -> Result<Vec<UserRecord>, AuthStoreError> {
        let rows = sqlx::query(
            "SELECT id, email, name, is_admin, access_enabled FROM users ORDER BY email",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(user_from_row).collect()
    }

    pub async fn set_access(&self, id: Uuid, enabled: bool) -> Result<(), AuthStoreError> {
        sqlx::query("UPDATE users SET access_enabled = ?2 WHERE id = ?1")
            .bind(id.to_string())
            .bind(enabled as i64)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn create_session(
        &self,
        user_id: Uuid,
        auth_kind: AuthKind,
        immich_cred: &[u8],
        server_epoch: i64,
        user_agent: Option<&str>,
        ip: Option<&str>,
    ) -> Result<String, AuthStoreError> {
        let prepared = self.prepare_session(immich_cred)?;
        let mut tx = self.pool.begin().await?;
        Self::insert_session(
            &mut tx,
            user_id,
            auth_kind,
            server_epoch,
            user_agent,
            ip,
            &prepared,
        )
        .await?;
        tx.commit().await?;
        Ok(prepared.token)
    }

    fn prepare_session(&self, immich_cred: &[u8]) -> Result<PreparedSession, AuthStoreError> {
        let mut token_bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut token_bytes);
        let token = URL_SAFE_NO_PAD.encode(token_bytes);
        let token_hash = hash_token(&token);
        let encrypted = self.crypto.encrypt(immich_cred)?;
        let now = Utc::now();
        Ok(PreparedSession {
            token,
            token_hash,
            encrypted,
            created_at: now.to_rfc3339(),
            expires_at: (now + Duration::days(IDLE_DAYS)).to_rfc3339(),
            absolute_expires_at: (now + Duration::days(ABSOLUTE_DAYS)).to_rfc3339(),
        })
    }

    async fn insert_session(
        tx: &mut Transaction<'_, Sqlite>,
        user_id: Uuid,
        auth_kind: AuthKind,
        server_epoch: i64,
        user_agent: Option<&str>,
        ip: Option<&str>,
        prepared: &PreparedSession,
    ) -> Result<(), AuthStoreError> {
        sqlx::query(
            "INSERT INTO sessions (id, user_id, token_hash, auth_kind, immich_cred_enc, \
             immich_cred_nonce, key_version, server_epoch, created_at, expires_at, \
             absolute_expires_at, last_seen_at, user_agent, ip) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?9, ?12, ?13)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(user_id.to_string())
        .bind(&prepared.token_hash)
        .bind(auth_kind.as_str())
        .bind(&prepared.encrypted.ciphertext)
        .bind(&prepared.encrypted.nonce)
        .bind(prepared.encrypted.key_version)
        .bind(server_epoch)
        .bind(&prepared.created_at)
        .bind(&prepared.expires_at)
        .bind(&prepared.absolute_expires_at)
        .bind(user_agent)
        .bind(ip)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    async fn insert_user(
        tx: &mut Transaction<'_, Sqlite>,
        user: &ImmichUser,
    ) -> Result<UserRecord, AuthStoreError> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO users (id, email, name, is_admin, access_enabled, created_at, last_login_at) \
             VALUES (?1, ?2, ?3, ?4, 1, ?5, ?5)",
        )
        .bind(user.id.to_string())
        .bind(&user.email)
        .bind(&user.name)
        .bind(user.is_admin as i64)
        .bind(&now)
        .execute(&mut **tx)
        .await?;
        Ok(UserRecord {
            id: user.id,
            email: user.email.clone(),
            name: user.name.clone(),
            is_admin: user.is_admin,
            access_enabled: true,
        })
    }

    pub async fn claim_instance_and_create_session(
        &self,
        immich_url: &str,
        user: &ImmichUser,
        auth_kind: AuthKind,
        immich_cred: &[u8],
        user_agent: Option<&str>,
        ip: Option<&str>,
    ) -> Result<(UserRecord, String), AuthStoreError> {
        let prepared = self.prepare_session(immich_cred)?;
        let now = Utc::now().to_rfc3339();
        let mut tx = self.pool.begin().await?;
        let result = sqlx::query(
            "UPDATE instance_config SET server_epoch = 1, immich_url = ?1, configured_at = ?2 \
             WHERE id = 1 AND server_epoch = 0",
        )
        .bind(immich_url)
        .bind(&now)
        .execute(&mut *tx)
        .await?;
        if result.rows_affected() == 0 {
            return Err(AuthStoreError::AlreadyConfigured);
        }
        let stored = Self::insert_user(&mut tx, user).await?;
        Self::insert_session(&mut tx, stored.id, auth_kind, 1, user_agent, ip, &prepared).await?;
        tx.commit().await?;
        Ok((stored, prepared.token))
    }

    pub async fn rebind_instance_and_create_session(
        &self,
        immich_url: &str,
        user: &ImmichUser,
        auth_kind: AuthKind,
        immich_cred: &[u8],
        user_agent: Option<&str>,
        ip: Option<&str>,
    ) -> Result<(UserRecord, String), AuthStoreError> {
        let prepared = self.prepare_session(immich_cred)?;
        let now = Utc::now().to_rfc3339();
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query(
            "UPDATE instance_config SET server_epoch = server_epoch + 1, immich_url = ?1, \
             configured_at = ?2 WHERE id = 1 RETURNING server_epoch",
        )
        .bind(immich_url)
        .bind(&now)
        .fetch_one(&mut *tx)
        .await?;
        let server_epoch: i64 = row.try_get("server_epoch")?;
        sqlx::query("DELETE FROM job_items")
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM job_credentials")
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM jobs").execute(&mut *tx).await?;
        sqlx::query("DELETE FROM edits").execute(&mut *tx).await?;
        sqlx::query("DELETE FROM edits_history")
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM presets").execute(&mut *tx).await?;
        sqlx::query("DELETE FROM export_jobs")
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM sessions")
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM users").execute(&mut *tx).await?;
        let stored = Self::insert_user(&mut tx, user).await?;
        Self::insert_session(
            &mut tx,
            stored.id,
            auth_kind,
            server_epoch,
            user_agent,
            ip,
            &prepared,
        )
        .await?;
        tx.commit().await?;
        Ok((stored, prepared.token))
    }

    pub async fn authenticate(&self, token: &str) -> Result<Option<AuthContext>, AuthStoreError> {
        let token_hash = hash_token(token);
        let now = Utc::now();
        let now_str = now.to_rfc3339();
        let row = sqlx::query(
            "SELECT s.id AS sid, s.user_id AS uid, s.auth_kind AS auth_kind, \
             s.server_epoch AS server_epoch, s.immich_cred_enc AS enc, s.immich_cred_nonce AS nonce, \
             s.key_version AS key_version, s.last_seen_at AS last_seen_at, \
             u.email AS email, u.name AS name, u.is_admin AS is_admin, u.access_enabled AS access_enabled \
             FROM sessions s JOIN users u ON u.id = s.user_id \
             JOIN instance_config i ON i.id = 1 AND i.server_epoch = s.server_epoch \
             WHERE s.token_hash = ?1 AND s.expires_at > ?2 AND s.absolute_expires_at > ?2",
        )
        .bind(token_hash)
        .bind(&now_str)
        .fetch_optional(&self.pool)
        .await?;
        let Some(row) = row else {
            return Ok(None);
        };
        let access_enabled: i64 = row.try_get("access_enabled")?;
        if access_enabled == 0 {
            return Ok(None);
        }
        let session_id = parse_uuid(row.try_get::<String, _>("sid")?)?;
        let user_id = parse_uuid(row.try_get::<String, _>("uid")?)?;
        let enc = Encrypted {
            ciphertext: row.try_get("enc")?,
            nonce: row.try_get("nonce")?,
            key_version: row.try_get("key_version")?,
        };
        let immich_cred = self.crypto.decrypt(&enc)?;
        let last_seen: String = row.try_get("last_seen_at")?;
        self.touch_session(session_id, &last_seen, now).await?;
        let user = UserRecord {
            id: user_id,
            email: row.try_get("email")?,
            name: row.try_get("name")?,
            is_admin: row.try_get::<i64, _>("is_admin")? != 0,
            access_enabled: true,
        };
        Ok(Some(AuthContext {
            user,
            session_id,
            server_epoch: row.try_get("server_epoch")?,
            auth_kind: AuthKind::from_wire(&row.try_get::<String, _>("auth_kind")?),
            immich_cred,
        }))
    }

    async fn touch_session(
        &self,
        session_id: Uuid,
        last_seen: &str,
        now: chrono::DateTime<Utc>,
    ) -> Result<(), AuthStoreError> {
        let stale = chrono::DateTime::parse_from_rfc3339(last_seen)
            .map(|t| (now - t.with_timezone(&Utc)).num_seconds() >= LAST_SEEN_THROTTLE_SECS)
            .unwrap_or(true);
        if !stale {
            return Ok(());
        }
        let expires = (now + Duration::days(IDLE_DAYS)).to_rfc3339();
        sqlx::query(
            "UPDATE sessions SET last_seen_at = ?2, expires_at = MIN(?3, absolute_expires_at) \
             WHERE id = ?1",
        )
        .bind(session_id.to_string())
        .bind(now.to_rfc3339())
        .bind(&expires)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn revoke_session(&self, id: Uuid) -> Result<bool, AuthStoreError> {
        let res = sqlx::query("DELETE FROM sessions WHERE id = ?1")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected() > 0)
    }

    pub async fn revoke_all_for_user(&self, user_id: Uuid) -> Result<(), AuthStoreError> {
        sqlx::query("DELETE FROM sessions WHERE user_id = ?1")
            .bind(user_id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn revoke_others_for_user(
        &self,
        user_id: Uuid,
        current_session_id: Uuid,
    ) -> Result<(), AuthStoreError> {
        sqlx::query("DELETE FROM sessions WHERE user_id = ?1 AND id != ?2")
            .bind(user_id.to_string())
            .bind(current_session_id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn purge_all(&self) -> Result<(), AuthStoreError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM sessions")
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM users").execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn list_sessions(&self, user_id: Uuid) -> Result<Vec<SessionRecord>, AuthStoreError> {
        let rows = sqlx::query(
            "SELECT id, user_id, auth_kind, server_epoch, created_at, expires_at, last_seen_at, \
             user_agent, ip FROM sessions WHERE user_id = ?1 ORDER BY last_seen_at DESC",
        )
        .bind(user_id.to_string())
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(session_from_row).collect()
    }

    pub async fn cleanup_expired(&self) -> Result<(), AuthStoreError> {
        let now = Utc::now().to_rfc3339();
        sqlx::query("DELETE FROM sessions WHERE expires_at <= ?1 OR absolute_expires_at <= ?1")
            .bind(now)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

fn user_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<UserRecord, AuthStoreError> {
    Ok(UserRecord {
        id: parse_uuid(row.try_get::<String, _>("id")?)?,
        email: row.try_get("email")?,
        name: row.try_get("name")?,
        is_admin: row.try_get::<i64, _>("is_admin")? != 0,
        access_enabled: row.try_get::<i64, _>("access_enabled")? != 0,
    })
}

fn session_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<SessionRecord, AuthStoreError> {
    Ok(SessionRecord {
        id: parse_uuid(row.try_get::<String, _>("id")?)?,
        user_id: parse_uuid(row.try_get::<String, _>("user_id")?)?,
        auth_kind: AuthKind::from_wire(&row.try_get::<String, _>("auth_kind")?),
        server_epoch: row.try_get("server_epoch")?,
        created_at: row.try_get("created_at")?,
        expires_at: row.try_get("expires_at")?,
        last_seen_at: row.try_get("last_seen_at")?,
        user_agent: row.try_get("user_agent")?,
        ip: row.try_get("ip")?,
    })
}

fn parse_uuid(s: String) -> Result<Uuid, AuthStoreError> {
    Uuid::parse_str(&s).map_err(|_| AuthStoreError::Db(sqlx::Error::RowNotFound))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::edits_store::EditsStore;

    async fn store() -> AuthStore {
        let edits = EditsStore::migrated_memory().await.unwrap();
        let dir = tempfile::tempdir().unwrap().keep();
        let crypto =
            Arc::new(InstanceCrypto::load_or_create(&dir.join("instance.key"), false).unwrap());
        AuthStore::new(edits.pool(), crypto)
    }

    fn immich_user(admin: bool) -> ImmichUser {
        ImmichUser {
            id: Uuid::new_v4(),
            email: "a@b.test".into(),
            name: "A".into(),
            is_admin: admin,
        }
    }

    #[tokio::test]
    async fn session_roundtrip_returns_context() {
        let s = store().await;
        sqlx::query("UPDATE instance_config SET server_epoch = 1 WHERE id = 1")
            .execute(&s.pool)
            .await
            .unwrap();
        let user = s.upsert_user(&immich_user(true)).await.unwrap();
        let token = s
            .create_session(user.id, AuthKind::Password, b"bearer-xyz", 1, None, None)
            .await
            .unwrap();
        let ctx = s.authenticate(&token).await.unwrap().unwrap();
        assert_eq!(ctx.user.id, user.id);
        assert!(ctx.user.is_admin);
        assert_eq!(ctx.immich_cred.as_slice(), b"bearer-xyz");
    }

    #[tokio::test]
    async fn unknown_token_rejected() {
        let s = store().await;
        assert!(s.authenticate("nope").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn disabled_user_denied() {
        let s = store().await;
        sqlx::query("UPDATE instance_config SET server_epoch = 1 WHERE id = 1")
            .execute(&s.pool)
            .await
            .unwrap();
        let user = s.upsert_user(&immich_user(false)).await.unwrap();
        let token = s
            .create_session(user.id, AuthKind::ApiKey, b"key", 1, None, None)
            .await
            .unwrap();
        s.set_access(user.id, false).await.unwrap();
        assert!(s.authenticate(&token).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn revoke_ends_session() {
        let s = store().await;
        sqlx::query("UPDATE instance_config SET server_epoch = 1 WHERE id = 1")
            .execute(&s.pool)
            .await
            .unwrap();
        let user = s.upsert_user(&immich_user(false)).await.unwrap();
        let token = s
            .create_session(user.id, AuthKind::Password, b"c", 1, None, None)
            .await
            .unwrap();
        let sessions = s.list_sessions(user.id).await.unwrap();
        assert_eq!(sessions.len(), 1);
        s.revoke_session(sessions[0].id).await.unwrap();
        assert!(s.authenticate(&token).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn upsert_preserves_access_flag() {
        let s = store().await;
        let iu = immich_user(false);
        let user = s.upsert_user(&iu).await.unwrap();
        s.set_access(user.id, false).await.unwrap();
        let again = s.upsert_user(&iu).await.unwrap();
        assert!(!again.access_enabled);
    }
}
