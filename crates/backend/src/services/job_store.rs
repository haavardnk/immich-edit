use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::services::auth_store::AuthKind;
use crate::services::crypto::{InstanceCrypto, SecretBytes};

#[derive(Debug, thiserror::Error)]
pub enum JobStoreError {
    #[error("db: {0}")]
    Db(#[from] sqlx::Error),
    #[error("parse: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("crypto: {0}")]
    Crypto(#[from] crate::services::crypto::CryptoError),
    #[error("corrupt row: {0}")]
    Corrupt(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl JobStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    fn from_str(s: &str) -> Result<Self, JobStoreError> {
        match s {
            "pending" => Ok(Self::Pending),
            "running" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            other => Err(JobStoreError::Corrupt(format!("job status {other}"))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobItemStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl JobItemStatus {
    fn from_str(s: &str) -> Result<Self, JobStoreError> {
        match s {
            "pending" => Ok(Self::Pending),
            "running" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            other => Err(JobStoreError::Corrupt(format!("job item status {other}"))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRecord {
    pub id: Uuid,
    #[serde(skip)]
    pub user_id: Uuid,
    #[serde(skip)]
    pub server_epoch: i64,
    #[serde(skip)]
    pub auth_session_id: Option<Uuid>,
    pub kind: String,
    pub status: JobStatus,
    pub target: serde_json::Value,
    pub params: serde_json::Value,
    pub total: i64,
    pub completed: i64,
    pub failed: i64,
    pub cancelled_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobItemRecord {
    pub id: Uuid,
    pub job_id: Uuid,
    pub asset_id: String,
    pub status: JobItemStatus,
    pub error: Option<String>,
    pub result: Option<serde_json::Value>,
    pub idempotency_key: Option<String>,
    pub attempts: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct NewJobItem {
    pub asset_id: String,
    pub idempotency_key: Option<String>,
}

#[derive(Clone)]
pub struct JobStore {
    pool: SqlitePool,
    crypto: Arc<InstanceCrypto>,
    events: broadcast::Sender<JobRecord>,
}

impl JobStore {
    pub fn new(pool: SqlitePool, crypto: Arc<InstanceCrypto>) -> Self {
        let (events, _) = broadcast::channel(256);
        Self {
            pool,
            crypto,
            events,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<JobRecord> {
        self.events.subscribe()
    }

    fn publish(&self, job: &JobRecord) {
        let _ = self.events.send(job.clone());
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_job(
        &self,
        owner: Uuid,
        server_epoch: i64,
        auth_session_id: Uuid,
        kind: &str,
        target: &serde_json::Value,
        params: &serde_json::Value,
        items: &[NewJobItem],
        cred: &[u8],
        auth_kind: AuthKind,
    ) -> Result<JobRecord, JobStoreError> {
        let id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();
        let target_json = serde_json::to_string(target)?;
        let params_json = serde_json::to_string(params)?;
        let total = items.len() as i64;
        let enc = self.crypto.encrypt(cred)?;

        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "INSERT INTO jobs (id, kind, status, target_json, params_json, total, completed, failed, created_at, updated_at, user_id, server_epoch, auth_session_id) \
             VALUES (?, ?, 'pending', ?, ?, ?, 0, 0, ?, ?, ?, ?, ?)",
        )
        .bind(id.to_string())
        .bind(kind)
        .bind(&target_json)
        .bind(&params_json)
        .bind(total)
        .bind(&now)
        .bind(&now)
        .bind(owner.to_string())
        .bind(server_epoch)
        .bind(auth_session_id.to_string())
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "INSERT INTO job_credentials (job_id, ciphertext, nonce, key_version, auth_kind) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .bind(id.to_string())
        .bind(enc.ciphertext)
        .bind(enc.nonce)
        .bind(enc.key_version)
        .bind(auth_kind.as_str())
        .execute(&mut *tx)
        .await?;

        for item in items {
            sqlx::query(
                "INSERT INTO job_items (id, job_id, asset_id, status, idempotency_key, attempts, created_at, updated_at) \
                 VALUES (?, ?, ?, 'pending', ?, 0, ?, ?)",
            )
            .bind(Uuid::new_v4().to_string())
            .bind(id.to_string())
            .bind(&item.asset_id)
            .bind(item.idempotency_key.as_deref())
            .bind(&now)
            .bind(&now)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;

        let job = self.get_job(id).await?.ok_or(sqlx::Error::RowNotFound)?;
        self.publish(&job);
        Ok(job)
    }

    pub async fn job_credential(
        &self,
        job_id: Uuid,
    ) -> Result<Option<(SecretBytes, AuthKind)>, JobStoreError> {
        let row = sqlx::query(
            "SELECT ciphertext, nonce, key_version, auth_kind FROM job_credentials WHERE job_id = ?1",
        )
        .bind(job_id.to_string())
        .fetch_optional(&self.pool)
        .await?;
        let Some(row) = row else {
            return Ok(None);
        };
        let enc = crate::services::crypto::Encrypted {
            ciphertext: row.get::<Vec<u8>, _>("ciphertext"),
            nonce: row.get::<Vec<u8>, _>("nonce"),
            key_version: row.get::<i64, _>("key_version"),
        };
        let cred = self.crypto.decrypt(&enc)?;
        let auth_kind = AuthKind::from_wire(&row.get::<String, _>("auth_kind"));
        Ok(Some((cred, auth_kind)))
    }

    pub async fn get_job(&self, id: Uuid) -> Result<Option<JobRecord>, JobStoreError> {
        let row = sqlx::query(
            "SELECT id, kind, status, target_json, params_json, total, completed, failed, cancelled_at, created_at, updated_at, user_id, server_epoch, auth_session_id \
             FROM jobs WHERE id = ?1",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;
        row.as_ref().map(job_from_row).transpose()
    }

    pub async fn list_jobs(
        &self,
        owner: Uuid,
        limit: i64,
    ) -> Result<Vec<JobRecord>, JobStoreError> {
        let rows = sqlx::query(
            "SELECT id, kind, status, target_json, params_json, total, completed, failed, cancelled_at, created_at, updated_at, user_id, server_epoch, auth_session_id \
             FROM jobs WHERE user_id = ?2 ORDER BY created_at DESC LIMIT ?1",
        )
        .bind(limit)
        .bind(owner.to_string())
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(job_from_row).collect()
    }

    pub async fn list_items(&self, job_id: Uuid) -> Result<Vec<JobItemRecord>, JobStoreError> {
        let rows = sqlx::query(
            "SELECT id, job_id, asset_id, status, error, result_json, idempotency_key, attempts, created_at, updated_at \
             FROM job_items WHERE job_id = ? ORDER BY created_at ASC",
        )
        .bind(job_id.to_string())
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(item_from_row).collect()
    }

    pub async fn claim_next_item(&self) -> Result<Option<JobItemRecord>, JobStoreError> {
        let now = Utc::now().to_rfc3339();
        let row = sqlx::query(
            "UPDATE job_items \
             SET status = 'running', attempts = attempts + 1, updated_at = ? \
             WHERE id = ( \
                 SELECT ji.id FROM job_items ji \
                 JOIN jobs j ON j.id = ji.job_id \
                 WHERE ji.status = 'pending' \
                   AND j.status IN ('pending', 'running') \
                   AND j.cancelled_at IS NULL \
                 ORDER BY ji.created_at ASC LIMIT 1 \
             ) \
             RETURNING id, job_id, asset_id, status, error, result_json, idempotency_key, attempts, created_at, updated_at",
        )
        .bind(&now)
        .fetch_optional(&self.pool)
        .await?;

        let item = row.as_ref().map(item_from_row).transpose()?;
        if let Some(item) = &item {
            let changed = sqlx::query(
                "UPDATE jobs SET status = 'running', updated_at = ? WHERE id = ? AND status = 'pending'",
            )
            .bind(&now)
            .bind(item.job_id.to_string())
            .execute(&self.pool)
            .await?;
            if changed.rows_affected() > 0
                && let Some(job) = self.get_job(item.job_id).await?
            {
                self.publish(&job);
            }
        }
        Ok(item)
    }

    pub async fn complete_item(
        &self,
        item_id: Uuid,
        result: &serde_json::Value,
    ) -> Result<(), JobStoreError> {
        let now = Utc::now().to_rfc3339();
        let result_json = serde_json::to_string(result)?;
        sqlx::query(
            "UPDATE job_items SET status = 'completed', result_json = ?, error = NULL, updated_at = ? WHERE id = ?",
        )
        .bind(&result_json)
        .bind(&now)
        .bind(item_id.to_string())
        .execute(&self.pool)
        .await?;
        self.finalize_for_item(item_id).await
    }

    pub async fn fail_item(&self, item_id: Uuid, error: &str) -> Result<(), JobStoreError> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE job_items SET status = 'failed', error = ?, updated_at = ? WHERE id = ?",
        )
        .bind(error)
        .bind(&now)
        .bind(item_id.to_string())
        .execute(&self.pool)
        .await?;
        self.finalize_for_item(item_id).await
    }

    async fn finalize_for_item(&self, item_id: Uuid) -> Result<(), JobStoreError> {
        let row = sqlx::query("SELECT job_id FROM job_items WHERE id = ?")
            .bind(item_id.to_string())
            .fetch_optional(&self.pool)
            .await?;
        let Some(row) = row else {
            return Ok(());
        };
        let job_id = parse_uuid(row.get::<String, _>("job_id"))?;
        self.recompute_and_finalize(job_id).await
    }

    async fn recompute_and_finalize(&self, job_id: Uuid) -> Result<(), JobStoreError> {
        let now = Utc::now().to_rfc3339();
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "UPDATE jobs SET \
                 completed = (SELECT COUNT(*) FROM job_items WHERE job_id = ?1 AND status = 'completed'), \
                 failed = (SELECT COUNT(*) FROM job_items WHERE job_id = ?1 AND status = 'failed'), \
                 updated_at = ?2 \
             WHERE id = ?1",
        )
        .bind(job_id.to_string())
        .bind(&now)
        .execute(&mut *tx)
        .await?;

        let row = sqlx::query(
            "SELECT \
                 (SELECT COUNT(*) FROM job_items WHERE job_id = ?1 AND status IN ('pending', 'running')) AS pending, \
                 (SELECT COUNT(*) FROM job_items WHERE job_id = ?1 AND status = 'completed') AS completed, \
                 (SELECT status FROM jobs WHERE id = ?1) AS status",
        )
        .bind(job_id.to_string())
        .fetch_one(&mut *tx)
        .await?;
        let pending: i64 = row.get("pending");
        let completed: i64 = row.get("completed");
        let status = JobStatus::from_str(&row.get::<String, _>("status"))?;

        if pending == 0 && matches!(status, JobStatus::Pending | JobStatus::Running) {
            let final_status = if completed > 0 {
                JobStatus::Completed
            } else {
                JobStatus::Failed
            };
            sqlx::query("UPDATE jobs SET status = ?, updated_at = ? WHERE id = ?")
                .bind(final_status.as_str())
                .bind(&now)
                .bind(job_id.to_string())
                .execute(&mut *tx)
                .await?;
            sqlx::query("DELETE FROM job_credentials WHERE job_id = ?1")
                .bind(job_id.to_string())
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;

        if let Some(job) = self.get_job(job_id).await? {
            self.publish(&job);
        }
        Ok(())
    }

    pub async fn cancel_job(&self, id: Uuid) -> Result<bool, JobStoreError> {
        let now = Utc::now().to_rfc3339();
        let mut tx = self.pool.begin().await?;
        let res = sqlx::query(
            "UPDATE jobs SET status = 'cancelled', cancelled_at = ?, updated_at = ? \
             WHERE id = ? AND status IN ('pending', 'running')",
        )
        .bind(&now)
        .bind(&now)
        .bind(id.to_string())
        .execute(&mut *tx)
        .await?;
        let cancelled = res.rows_affected() > 0;
        if cancelled {
            sqlx::query("DELETE FROM job_credentials WHERE job_id = ?1")
                .bind(id.to_string())
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        if cancelled && let Some(job) = self.get_job(id).await? {
            self.publish(&job);
        }
        Ok(cancelled)
    }

    pub async fn cancel_active_for_owner(&self, owner: Uuid) -> Result<u64, JobStoreError> {
        let now = Utc::now().to_rfc3339();
        let mut tx = self.pool.begin().await?;
        let res = sqlx::query(
            "UPDATE jobs SET status = 'cancelled', cancelled_at = ?1, updated_at = ?1 \
             WHERE user_id = ?2 AND status IN ('pending', 'running')",
        )
        .bind(&now)
        .bind(owner.to_string())
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            "DELETE FROM job_credentials WHERE job_id IN \
             (SELECT id FROM jobs WHERE user_id = ?1 AND status = 'cancelled')",
        )
        .bind(owner.to_string())
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(res.rows_affected())
    }

    pub async fn cancel_active_for_session(&self, session_id: Uuid) -> Result<u64, JobStoreError> {
        let now = Utc::now().to_rfc3339();
        let session = session_id.to_string();
        let mut tx = self.pool.begin().await?;
        let result = sqlx::query(
            "UPDATE jobs SET status = 'cancelled', cancelled_at = ?1, updated_at = ?1 \
             WHERE auth_session_id = ?2 AND status IN ('pending', 'running')",
        )
        .bind(&now)
        .bind(&session)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            "DELETE FROM job_credentials WHERE job_id IN \
             (SELECT id FROM jobs WHERE auth_session_id = ?1 AND status = 'cancelled')",
        )
        .bind(&session)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(result.rows_affected())
    }

    pub async fn cancel_active_for_other_sessions(
        &self,
        owner: Uuid,
        current_session_id: Uuid,
    ) -> Result<u64, JobStoreError> {
        let now = Utc::now().to_rfc3339();
        let owner = owner.to_string();
        let current = current_session_id.to_string();
        let mut tx = self.pool.begin().await?;
        let result = sqlx::query(
            "UPDATE jobs SET status = 'cancelled', cancelled_at = ?1, updated_at = ?1 \
             WHERE user_id = ?2 AND auth_session_id != ?3 AND status IN ('pending', 'running')",
        )
        .bind(&now)
        .bind(&owner)
        .bind(&current)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            "DELETE FROM job_credentials WHERE job_id IN \
             (SELECT id FROM jobs WHERE user_id = ?1 AND auth_session_id != ?2 AND status = 'cancelled')",
        )
        .bind(&owner)
        .bind(&current)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(result.rows_affected())
    }

    pub async fn purge_owner(&self, owner: Uuid) -> Result<(), JobStoreError> {
        let owner_str = owner.to_string();
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "DELETE FROM job_items WHERE job_id IN (SELECT id FROM jobs WHERE user_id = ?1)",
        )
        .bind(&owner_str)
        .execute(&mut *tx)
        .await?;
        sqlx::query("DELETE FROM jobs WHERE user_id = ?1")
            .bind(&owner_str)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn purge_all(&self) -> Result<(), JobStoreError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM job_items")
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM jobs").execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn clear_finished(&self, owner: Uuid) -> Result<Vec<(Uuid, String)>, JobStoreError> {
        let owner_str = owner.to_string();
        let rows = sqlx::query(
            "SELECT id, kind FROM jobs WHERE user_id = ?1 AND status NOT IN ('pending', 'running')",
        )
        .bind(&owner_str)
        .fetch_all(&self.pool)
        .await?;
        let cleared: Vec<(Uuid, String)> = rows
            .iter()
            .map(|r| Ok((parse_uuid(r.get("id"))?, r.get::<String, _>("kind"))))
            .collect::<Result<_, JobStoreError>>()?;
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "DELETE FROM job_items WHERE job_id IN (SELECT id FROM jobs WHERE user_id = ?1 AND status NOT IN ('pending', 'running'))",
        )
        .bind(&owner_str)
        .execute(&mut *tx)
        .await?;
        sqlx::query("DELETE FROM jobs WHERE user_id = ?1 AND status NOT IN ('pending', 'running')")
            .bind(&owner_str)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(cleared)
    }

    pub async fn requeue_running(&self) -> Result<u64, JobStoreError> {
        let now = Utc::now().to_rfc3339();
        let items = sqlx::query(
            "UPDATE job_items SET status = 'pending', updated_at = ? WHERE status = 'running'",
        )
        .bind(&now)
        .execute(&self.pool)
        .await?;
        sqlx::query("UPDATE jobs SET status = 'pending', updated_at = ? WHERE status = 'running'")
            .bind(&now)
            .execute(&self.pool)
            .await?;
        Ok(items.rows_affected())
    }
}

fn parse_uuid(s: String) -> Result<Uuid, JobStoreError> {
    Uuid::parse_str(&s).map_err(|_| JobStoreError::Corrupt(format!("uuid {s}")))
}

fn job_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<JobRecord, JobStoreError> {
    Ok(JobRecord {
        id: parse_uuid(row.get("id"))?,
        user_id: parse_uuid(row.get("user_id"))?,
        server_epoch: row.get("server_epoch"),
        auth_session_id: row
            .get::<Option<String>, _>("auth_session_id")
            .map(parse_uuid)
            .transpose()?,
        kind: row.get("kind"),
        status: JobStatus::from_str(&row.get::<String, _>("status"))?,
        target: serde_json::from_str(&row.get::<String, _>("target_json"))?,
        params: serde_json::from_str(&row.get::<String, _>("params_json"))?,
        total: row.get("total"),
        completed: row.get("completed"),
        failed: row.get("failed"),
        cancelled_at: row.get("cancelled_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

fn item_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<JobItemRecord, JobStoreError> {
    let result = match row.get::<Option<String>, _>("result_json") {
        Some(s) => Some(serde_json::from_str(&s)?),
        None => None,
    };
    Ok(JobItemRecord {
        id: parse_uuid(row.get("id"))?,
        job_id: parse_uuid(row.get("job_id"))?,
        asset_id: row.get("asset_id"),
        status: JobItemStatus::from_str(&row.get::<String, _>("status"))?,
        error: row.get("error"),
        result,
        idempotency_key: row.get("idempotency_key"),
        attempts: row.get("attempts"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::crypto::InstanceCrypto;
    use crate::services::edits_store::EditsStore;
    use serde_json::json;

    async fn store() -> JobStore {
        let edits = EditsStore::migrated_memory().await.expect("memory store");
        let dir = tempfile::tempdir().unwrap();
        let crypto = std::sync::Arc::new(
            InstanceCrypto::load_or_create(&dir.path().join("instance.key"), false).unwrap(),
        );
        std::mem::forget(dir);
        JobStore::new(edits.pool(), crypto)
    }

    fn items(ids: &[&str]) -> Vec<NewJobItem> {
        ids.iter()
            .map(|id| NewJobItem {
                asset_id: (*id).to_string(),
                idempotency_key: None,
            })
            .collect()
    }

    async fn create(
        store: &JobStore,
        owner: Uuid,
        kind: &str,
        target: &serde_json::Value,
        params: &serde_json::Value,
        items: &[NewJobItem],
    ) -> JobRecord {
        store
            .create_job(
                owner,
                1,
                Uuid::nil(),
                kind,
                target,
                params,
                items,
                b"test-key",
                AuthKind::ApiKey,
            )
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn create_and_claim_drains_items() {
        let store = store().await;
        let job = create(
            &store,
            Uuid::nil(),
            "test",
            &json!(null),
            &json!(null),
            &items(&["a", "b"]),
        )
        .await;
        assert_eq!(job.total, 2);
        assert_eq!(job.status, JobStatus::Pending);

        let first = store.claim_next_item().await.unwrap().unwrap();
        assert_eq!(first.status, JobItemStatus::Running);
        assert_eq!(first.attempts, 1);
        store
            .complete_item(first.id, &json!({"ok": true}))
            .await
            .unwrap();

        let second = store.claim_next_item().await.unwrap().unwrap();
        store.fail_item(second.id, "boom").await.unwrap();

        assert!(store.claim_next_item().await.unwrap().is_none());

        let done = store.get_job(job.id).await.unwrap().unwrap();
        assert_eq!(done.status, JobStatus::Completed);
        assert_eq!(done.completed, 1);
        assert_eq!(done.failed, 1);
    }

    #[tokio::test]
    async fn cancel_blocks_further_claims() {
        let store = store().await;
        let job = create(
            &store,
            Uuid::nil(),
            "test",
            &json!(null),
            &json!(null),
            &items(&["a", "b"]),
        )
        .await;
        assert!(store.cancel_job(job.id).await.unwrap());
        assert!(store.claim_next_item().await.unwrap().is_none());

        let cancelled = store.get_job(job.id).await.unwrap().unwrap();
        assert_eq!(cancelled.status, JobStatus::Cancelled);
        assert!(cancelled.cancelled_at.is_some());
        assert!(!store.cancel_job(job.id).await.unwrap());
    }

    #[tokio::test]
    async fn requeue_resets_running_state() {
        let store = store().await;
        let job = create(
            &store,
            Uuid::nil(),
            "test",
            &json!(null),
            &json!(null),
            &items(&["a"]),
        )
        .await;
        let claimed = store.claim_next_item().await.unwrap().unwrap();
        assert_eq!(claimed.status, JobItemStatus::Running);

        let requeued = store.requeue_running().await.unwrap();
        assert_eq!(requeued, 1);

        let job = store.get_job(job.id).await.unwrap().unwrap();
        assert_eq!(job.status, JobStatus::Pending);
        let again = store.claim_next_item().await.unwrap().unwrap();
        assert_eq!(again.attempts, 2);
    }

    #[tokio::test]
    async fn corrupt_rows_are_rejected() {
        let cases = [
            (
                "UPDATE jobs SET user_id = 'not-a-uuid' WHERE id = ?",
                "uuid",
            ),
            (
                "UPDATE jobs SET status = 'weird' WHERE id = ?",
                "job status",
            ),
        ];
        for (sql, expected) in cases {
            let store = store().await;
            let job = create(
                &store,
                Uuid::nil(),
                "test",
                &json!(null),
                &json!(null),
                &items(&["a"]),
            )
            .await;
            sqlx::query(sql)
                .bind(job.id.to_string())
                .execute(&store.pool)
                .await
                .unwrap();
            let err = store.get_job(job.id).await.unwrap_err();
            match err {
                JobStoreError::Corrupt(msg) => assert!(msg.contains(expected), "{msg}"),
                other => panic!("expected corrupt error, got {other}"),
            }
        }
    }
}
