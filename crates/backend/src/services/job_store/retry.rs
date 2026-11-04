use chrono::Utc;
use uuid::Uuid;

use super::credentials::delete_credential;
use super::*;

impl JobStore {
    pub async fn retry_failed(
        &self,
        job_id: Uuid,
        auth_session_id: Uuid,
        cred: &[u8],
        auth_kind: AuthKind,
    ) -> Result<Option<JobRecord>, JobStoreError> {
        let now = Utc::now().to_rfc3339();
        let mut tx = self.pool.begin().await?;
        let reopened = sqlx::query(
            "UPDATE jobs SET status = 'pending', failed = 0, updated_at = ?1, auth_session_id = ?2 \
             WHERE id = ?3 AND status IN ('completed', 'failed') AND failed > 0",
        )
        .bind(&now)
        .bind(auth_session_id.to_string())
        .bind(job_id.to_string())
        .execute(&mut *tx)
        .await?;
        if reopened.rows_affected() == 0 {
            return Ok(None);
        }
        sqlx::query(
            "UPDATE job_items SET status = 'pending', error = NULL, result_json = NULL, updated_at = ?1 \
             WHERE job_id = ?2 AND status = 'failed'",
        )
        .bind(&now)
        .bind(job_id.to_string())
        .execute(&mut *tx)
        .await?;
        delete_credential(&mut tx, job_id).await?;
        self.insert_credential(&mut tx, job_id, cred, auth_kind)
            .await?;
        tx.commit().await?;

        let job = self
            .get_job(job_id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?;
        self.publish(&job);
        Ok(Some(job))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::job_store::tests::{create, store};
    use serde_json::json;

    async fn settle_next(store: &JobStore, succeed: bool) -> JobItemRecord {
        let item = store.claim_next_item().await.unwrap().unwrap();
        if succeed {
            store
                .complete_item(item.id, &json!({"ok": true}))
                .await
                .unwrap();
        } else {
            store.fail_item(item.id, "boom").await.unwrap();
        }
        item
    }

    async fn finished_with_one_failure(store: &JobStore) -> JobRecord {
        let job = create(store, &["ok", "bad"]).await;
        settle_next(store, true).await;
        settle_next(store, false).await;
        store.get_job(job.id).await.unwrap().unwrap()
    }

    async fn retry(store: &JobStore, job_id: Uuid) -> Option<JobRecord> {
        store
            .retry_failed(job_id, Uuid::nil(), b"test-key", AuthKind::ApiKey)
            .await
            .unwrap()
    }

    async fn assert_not_retried(store: &JobStore, job_id: Uuid, status: JobStatus) {
        let item_states = |items: Vec<JobItemRecord>| -> Vec<(JobItemStatus, Option<String>)> {
            items
                .into_iter()
                .map(|item| (item.status, item.error))
                .collect()
        };
        let before = item_states(store.list_items(job_id).await.unwrap());

        assert!(retry(store, job_id).await.is_none(), "{status:?}");

        assert_eq!(store.get_job(job_id).await.unwrap().unwrap().status, status);
        assert_eq!(item_states(store.list_items(job_id).await.unwrap()), before);
    }

    #[tokio::test]
    async fn retry_reopens_only_the_failed_items() {
        let store = store().await;
        let job = finished_with_one_failure(&store).await;
        assert_eq!(job.status, JobStatus::Completed);
        let session = Uuid::new_v4();

        let retried = store
            .retry_failed(job.id, session, b"test-key", AuthKind::ApiKey)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(retried.status, JobStatus::Pending);
        assert_eq!((retried.completed, retried.failed), (1, 0));
        let listed = store.list_items(job.id).await.unwrap();
        let [ok, bad] = listed.as_slice() else {
            panic!("expected two items, got {listed:?}");
        };
        assert_eq!(ok.status, JobItemStatus::Completed);
        assert_eq!(ok.result, Some(json!({"ok": true})));
        assert_eq!(bad.status, JobItemStatus::Pending);
        assert_eq!((bad.error.as_deref(), bad.result.as_ref()), (None, None));
        assert_eq!((bad.attempts, bad.position), (1, 2));
        let stored_session: String =
            sqlx::query_scalar("SELECT auth_session_id FROM jobs WHERE id = ?")
                .bind(job.id.to_string())
                .fetch_one(&store.pool)
                .await
                .unwrap();
        assert_eq!(stored_session, session.to_string());

        let reclaimed = settle_next(&store, true).await;
        assert_eq!((reclaimed.id, reclaimed.attempts), (bad.id, 2));
        let settled = store.get_job(job.id).await.unwrap().unwrap();
        assert_eq!(settled.status, JobStatus::Completed);
        assert_eq!((settled.completed, settled.failed), (2, 0));
    }

    #[tokio::test]
    async fn retry_stores_the_callers_credential() {
        let store = store().await;
        let job = finished_with_one_failure(&store).await;
        assert!(store.job_credential(job.id).await.unwrap().is_none());

        store
            .retry_failed(job.id, Uuid::nil(), b"fresh-key", AuthKind::Password)
            .await
            .unwrap()
            .unwrap();

        let (cred, kind) = store.job_credential(job.id).await.unwrap().unwrap();
        assert_eq!(cred.to_utf8().as_deref(), Some("fresh-key"));
        assert_eq!(kind, AuthKind::Password);
    }

    #[tokio::test]
    async fn retry_reopens_a_job_where_every_item_failed() {
        let store = store().await;
        let job = create(&store, &["bad"]).await;
        settle_next(&store, false).await;
        assert_eq!(
            store.get_job(job.id).await.unwrap().unwrap().status,
            JobStatus::Failed
        );

        let retried = retry(&store, job.id).await.unwrap();

        assert_eq!(retried.status, JobStatus::Pending);
        assert_eq!(retried.failed, 0);
    }

    #[tokio::test]
    async fn retry_publishes_the_reopened_job() {
        let store = store().await;
        let job = finished_with_one_failure(&store).await;
        let mut events = store.subscribe();

        retry(&store, job.id).await.unwrap();

        let published = events.recv().await.unwrap();
        assert_eq!(published.id, job.id);
        assert_eq!(published.status, JobStatus::Pending);
    }

    #[tokio::test]
    async fn retry_skips_a_running_job() {
        let store = store().await;
        let job = create(&store, &["a", "b"]).await;
        settle_next(&store, false).await;

        assert_not_retried(&store, job.id, JobStatus::Running).await;
    }

    #[tokio::test]
    async fn retry_skips_a_pending_job() {
        let store = store().await;
        let job = create(&store, &["a", "b"]).await;
        settle_next(&store, false).await;
        store.claim_next_item().await.unwrap().unwrap();
        store.requeue_running().await.unwrap();

        assert_not_retried(&store, job.id, JobStatus::Pending).await;
    }

    #[tokio::test]
    async fn retry_skips_a_cancelled_job() {
        let store = store().await;
        let job = create(&store, &["a", "b"]).await;
        settle_next(&store, false).await;
        assert!(store.cancel_job(job.id).await.unwrap());

        assert_not_retried(&store, job.id, JobStatus::Cancelled).await;
    }

    #[tokio::test]
    async fn retry_skips_a_job_without_failures() {
        let store = store().await;
        let job = create(&store, &["a"]).await;
        settle_next(&store, true).await;

        assert_not_retried(&store, job.id, JobStatus::Completed).await;
    }

    #[tokio::test]
    async fn a_retried_job_survives_clearing_finished_jobs() {
        let store = store().await;
        let finished = create(&store, &["c"]).await;
        settle_next(&store, true).await;
        let retried = finished_with_one_failure(&store).await;
        retry(&store, retried.id).await.unwrap();

        let cleared = store.clear_finished(Uuid::nil()).await.unwrap();

        assert_eq!(cleared, [(finished.id, "test".to_string())]);
        assert!(store.get_job(retried.id).await.unwrap().is_some());
        assert_eq!(store.list_items(retried.id).await.unwrap().len(), 2);
        assert!(store.get_job(finished.id).await.unwrap().is_none());
        assert!(store.list_items(finished.id).await.unwrap().is_empty());
    }
}
