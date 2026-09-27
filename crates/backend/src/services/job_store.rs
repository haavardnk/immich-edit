use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;
use uuid::fmt::Hyphenated;

mod cancel;
mod credentials;
mod finalize;

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(rename_all = "lowercase")]
pub enum JobItemStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct JobRecord {
    #[sqlx(try_from = "Hyphenated")]
    pub id: Uuid,
    #[serde(skip)]
    #[sqlx(try_from = "Hyphenated")]
    pub user_id: Uuid,
    #[serde(skip)]
    pub server_epoch: i64,
    pub kind: String,
    pub status: JobStatus,
    #[sqlx(rename = "target_json", json)]
    pub target: serde_json::Value,
    #[sqlx(rename = "params_json", json)]
    pub params: serde_json::Value,
    pub total: i64,
    pub completed: i64,
    pub failed: i64,
    pub cancelled_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct JobItemRecord {
    #[sqlx(try_from = "Hyphenated")]
    pub id: Uuid,
    #[sqlx(try_from = "Hyphenated")]
    pub job_id: Uuid,
    pub asset_id: String,
    pub status: JobItemStatus,
    pub error: Option<String>,
    #[sqlx(rename = "result_json", json(nullable))]
    pub result: Option<serde_json::Value>,
    pub idempotency_key: Option<String>,
    pub attempts: i64,
    pub position: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct NewJobItem {
    pub asset_id: String,
    pub idempotency_key: Option<String>,
}

pub struct NewJob<'a> {
    pub owner: Uuid,
    pub server_epoch: i64,
    pub auth_session_id: Uuid,
    pub kind: &'a str,
    pub target: &'a serde_json::Value,
    pub params: &'a serde_json::Value,
    pub items: &'a [NewJobItem],
    pub cred: &'a [u8],
    pub auth_kind: AuthKind,
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

    pub async fn create_job(&self, job: NewJob<'_>) -> Result<JobRecord, JobStoreError> {
        let NewJob {
            owner,
            server_epoch,
            auth_session_id,
            kind,
            target,
            params,
            items,
            cred,
            auth_kind,
        } = job;
        let id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();
        let target_json = serde_json::to_string(target)?;
        let params_json = serde_json::to_string(params)?;
        let total = items.len() as i64;

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

        self.insert_credential(&mut tx, id, cred, auth_kind).await?;

        for (position, item) in (1_i64..).zip(items) {
            sqlx::query(
                "INSERT INTO job_items (id, job_id, asset_id, status, idempotency_key, attempts, position, created_at, updated_at) \
                 VALUES (?, ?, ?, 'pending', ?, 0, ?, ?, ?)",
            )
            .bind(Uuid::new_v4().to_string())
            .bind(id.to_string())
            .bind(&item.asset_id)
            .bind(item.idempotency_key.as_deref())
            .bind(position)
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

    pub async fn get_job(&self, id: Uuid) -> Result<Option<JobRecord>, JobStoreError> {
        Ok(sqlx::query_as::<_, JobRecord>(
            "SELECT id, kind, status, target_json, params_json, total, completed, failed, cancelled_at, created_at, updated_at, user_id, server_epoch \
             FROM jobs WHERE id = ?1",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?)
    }

    pub async fn list_jobs(
        &self,
        owner: Uuid,
        limit: i64,
    ) -> Result<Vec<JobRecord>, JobStoreError> {
        Ok(sqlx::query_as::<_, JobRecord>(
            "SELECT id, kind, status, target_json, params_json, total, completed, failed, cancelled_at, created_at, updated_at, user_id, server_epoch \
             FROM jobs WHERE user_id = ?2 ORDER BY created_at DESC LIMIT ?1",
        )
        .bind(limit)
        .bind(owner.to_string())
        .fetch_all(&self.pool)
        .await?)
    }

    pub async fn list_items(&self, job_id: Uuid) -> Result<Vec<JobItemRecord>, JobStoreError> {
        Ok(sqlx::query_as::<_, JobItemRecord>(
            "SELECT id, job_id, asset_id, status, error, result_json, idempotency_key, attempts, position, created_at, updated_at \
             FROM job_items WHERE job_id = ? ORDER BY position ASC, created_at ASC",
        )
        .bind(job_id.to_string())
        .fetch_all(&self.pool)
        .await?)
    }

    pub async fn claim_next_item(&self) -> Result<Option<JobItemRecord>, JobStoreError> {
        let now = Utc::now().to_rfc3339();
        let item = sqlx::query_as::<_, JobItemRecord>(
            "UPDATE job_items \
             SET status = 'running', attempts = attempts + 1, updated_at = ? \
             WHERE id = ( \
                 SELECT ji.id FROM job_items ji \
                 JOIN jobs j ON j.id = ji.job_id \
                 WHERE ji.status = 'pending' \
                   AND j.status IN ('pending', 'running') \
                   AND j.cancelled_at IS NULL \
                 ORDER BY ji.created_at ASC, ji.position ASC LIMIT 1 \
             ) \
             RETURNING id, job_id, asset_id, status, error, result_json, idempotency_key, attempts, position, created_at, updated_at",
        )
        .bind(&now)
        .fetch_optional(&self.pool)
        .await?;

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

    pub async fn clear_finished(&self, owner: Uuid) -> Result<Vec<(Uuid, String)>, JobStoreError> {
        let owner_str = owner.to_string();
        let cleared: Vec<(Uuid, String)> = sqlx::query_as::<_, (Hyphenated, String)>(
            "SELECT id, kind FROM jobs WHERE user_id = ?1 AND status NOT IN ('pending', 'running')",
        )
        .bind(&owner_str)
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|(id, kind)| (id.into_uuid(), kind))
        .collect();
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
            .create_job(NewJob {
                owner,
                server_epoch: 1,
                auth_session_id: Uuid::nil(),
                kind,
                target,
                params,
                items,
                cred: b"test-key",
                auth_kind: AuthKind::ApiKey,
            })
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
    async fn items_keep_their_selection_order() {
        let store = store().await;
        let job = create(
            &store,
            Uuid::nil(),
            "test",
            &json!(null),
            &json!(null),
            &items(&["c", "a", "b"]),
        )
        .await;
        let listed: Vec<(String, i64)> = store
            .list_items(job.id)
            .await
            .unwrap()
            .into_iter()
            .map(|item| (item.asset_id, item.position))
            .collect();
        assert_eq!(listed, [("c".into(), 1), ("a".into(), 2), ("b".into(), 3)]);
        let claimed = store.claim_next_item().await.unwrap().unwrap();
        assert_eq!((claimed.asset_id.as_str(), claimed.position), ("c", 1));
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
    async fn credentials_are_dropped_when_a_job_settles() {
        for cancel in [false, true] {
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
            assert!(store.job_credential(job.id).await.unwrap().is_some());

            if cancel {
                store.cancel_job(job.id).await.unwrap();
            } else {
                let claimed = store.claim_next_item().await.unwrap().unwrap();
                store
                    .complete_item(claimed.id, &json!({"ok": true}))
                    .await
                    .unwrap();
            }

            assert!(
                store.job_credential(job.id).await.unwrap().is_none(),
                "cancel={cancel}"
            );
        }
    }

    #[tokio::test]
    async fn corrupt_rows_are_rejected() {
        let cases = [
            (
                "UPDATE jobs SET user_id = 'not-a-uuid' WHERE id = ?",
                "user_id",
            ),
            ("UPDATE jobs SET status = 'weird' WHERE id = ?", "status"),
            (
                "UPDATE jobs SET target_json = '{' WHERE id = ?",
                "target_json",
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
                JobStoreError::Db(sqlx::Error::ColumnDecode { index, .. }) => {
                    assert!(index.contains(expected), "{index}")
                }
                other => panic!("expected column decode error, got {other}"),
            }
        }
    }

    #[tokio::test]
    async fn statuses_are_stored_as_lowercase_names() {
        let store = store().await;
        let jobs = [
            (JobStatus::Pending, "pending"),
            (JobStatus::Running, "running"),
            (JobStatus::Completed, "completed"),
            (JobStatus::Failed, "failed"),
            (JobStatus::Cancelled, "cancelled"),
        ];
        for (status, name) in jobs {
            let raw: String = sqlx::query_scalar("SELECT ?")
                .bind(status)
                .fetch_one(&store.pool)
                .await
                .unwrap();
            assert_eq!(raw, name);
            let back: JobStatus = sqlx::query_scalar("SELECT ?")
                .bind(name)
                .fetch_one(&store.pool)
                .await
                .unwrap();
            assert_eq!(back, status);
        }
        let items = [
            (JobItemStatus::Pending, "pending"),
            (JobItemStatus::Running, "running"),
            (JobItemStatus::Completed, "completed"),
            (JobItemStatus::Failed, "failed"),
        ];
        for (status, name) in items {
            let raw: String = sqlx::query_scalar("SELECT ?")
                .bind(status)
                .fetch_one(&store.pool)
                .await
                .unwrap();
            assert_eq!(raw, name);
        }
    }
}
