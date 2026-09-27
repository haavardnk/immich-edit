use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{Duration, Utc};
use rand::Rng;
use sha2::{Digest, Sha256};
use sqlx::{Sqlite, Transaction};

use super::*;
use crate::services::crypto::Encrypted;

const IDLE_DAYS: i64 = 30;
const ABSOLUTE_DAYS: i64 = 90;
const LAST_SEEN_THROTTLE_SECS: i64 = 300;

#[derive(sqlx::FromRow)]
struct AuthRow {
    #[sqlx(flatten)]
    user: UserRecord,
    #[sqlx(try_from = "Hyphenated")]
    sid: Uuid,
    auth_kind: AuthKind,
    server_epoch: i64,
    enc: Vec<u8>,
    nonce: Vec<u8>,
    key_version: i64,
    last_seen_at: String,
}

pub(super) struct PreparedSession {
    pub(super) token: String,
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

    pub(super) fn prepare_session(
        &self,
        immich_cred: &[u8],
    ) -> Result<PreparedSession, AuthStoreError> {
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

    pub(super) async fn insert_session(
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
        .bind(auth_kind)
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

    pub async fn authenticate(&self, token: &str) -> Result<Option<AuthContext>, AuthStoreError> {
        let token_hash = hash_token(token);
        let now = Utc::now();
        let now_str = now.to_rfc3339();
        let row = sqlx::query_as::<_, AuthRow>(
            "SELECT s.id AS sid, s.auth_kind AS auth_kind, \
             s.server_epoch AS server_epoch, s.immich_cred_enc AS enc, s.immich_cred_nonce AS nonce, \
             s.key_version AS key_version, s.last_seen_at AS last_seen_at, \
             u.id AS id, u.email AS email, u.name AS name, u.is_admin AS is_admin, \
             u.access_enabled AS access_enabled \
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
        if !row.user.access_enabled {
            return Ok(None);
        }
        let immich_cred = self.crypto.decrypt(&Encrypted {
            ciphertext: row.enc,
            nonce: row.nonce,
            key_version: row.key_version,
        })?;
        self.touch_session(row.sid, &row.last_seen_at, now).await?;
        Ok(Some(AuthContext {
            user: row.user,
            session_id: row.sid,
            server_epoch: row.server_epoch,
            auth_kind: row.auth_kind,
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

    pub async fn list_sessions(&self, user_id: Uuid) -> Result<Vec<SessionRecord>, AuthStoreError> {
        Ok(sqlx::query_as::<_, SessionRecord>(
            "SELECT id, user_id, auth_kind, server_epoch, created_at, expires_at, last_seen_at, \
             user_agent, ip FROM sessions WHERE user_id = ?1 ORDER BY last_seen_at DESC",
        )
        .bind(user_id.to_string())
        .fetch_all(&self.pool)
        .await?)
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
