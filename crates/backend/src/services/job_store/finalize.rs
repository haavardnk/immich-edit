use chrono::Utc;
use uuid::Uuid;

use super::credentials::delete_credential;
use super::*;

impl JobStore {
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
        let job_id: Option<Hyphenated> =
            sqlx::query_scalar("SELECT job_id FROM job_items WHERE id = ?")
                .bind(item_id.to_string())
                .fetch_optional(&self.pool)
                .await?;
        let Some(job_id) = job_id else {
            return Ok(());
        };
        self.recompute_and_finalize(job_id.into_uuid()).await
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
        let pending: i64 = row.try_get("pending")?;
        let completed: i64 = row.try_get("completed")?;
        let status: JobStatus = row.try_get("status")?;

        if pending == 0 && matches!(status, JobStatus::Pending | JobStatus::Running) {
            let final_status = if completed > 0 {
                JobStatus::Completed
            } else {
                JobStatus::Failed
            };
            delete_credential(&mut tx, job_id).await?;
            sqlx::query("UPDATE jobs SET status = ?, updated_at = ? WHERE id = ?")
                .bind(final_status)
                .bind(&now)
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
}
