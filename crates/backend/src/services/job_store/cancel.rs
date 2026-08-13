use chrono::Utc;
use uuid::Uuid;

use super::*;

impl JobStore {
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

    async fn cancel_where(
        &self,
        update: &'static str,
        delete: &'static str,
        binds: &[String],
    ) -> Result<u64, JobStoreError> {
        let now = Utc::now().to_rfc3339();
        let mut tx = self.pool.begin().await?;
        let mut update_query = sqlx::query(update);
        for bind in binds {
            update_query = update_query.bind(bind.as_str());
        }
        let result = update_query.bind(&now).execute(&mut *tx).await?;
        let mut delete_query = sqlx::query(delete);
        for bind in binds {
            delete_query = delete_query.bind(bind.as_str());
        }
        delete_query.execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(result.rows_affected())
    }

    pub async fn cancel_active_for_owner(&self, owner: Uuid) -> Result<u64, JobStoreError> {
        self.cancel_where(
            "UPDATE jobs SET status = 'cancelled', cancelled_at = ?2, updated_at = ?2 \
             WHERE user_id = ?1 AND status IN ('pending', 'running')",
            "DELETE FROM job_credentials WHERE job_id IN \
             (SELECT id FROM jobs WHERE user_id = ?1 AND status = 'cancelled')",
            &[owner.to_string()],
        )
        .await
    }

    pub async fn cancel_active_for_session(&self, session_id: Uuid) -> Result<u64, JobStoreError> {
        self.cancel_where(
            "UPDATE jobs SET status = 'cancelled', cancelled_at = ?2, updated_at = ?2 \
             WHERE auth_session_id = ?1 AND status IN ('pending', 'running')",
            "DELETE FROM job_credentials WHERE job_id IN \
             (SELECT id FROM jobs WHERE auth_session_id = ?1 AND status = 'cancelled')",
            &[session_id.to_string()],
        )
        .await
    }

    pub async fn cancel_active_for_other_sessions(
        &self,
        owner: Uuid,
        current_session_id: Uuid,
    ) -> Result<u64, JobStoreError> {
        self.cancel_where(
            "UPDATE jobs SET status = 'cancelled', cancelled_at = ?3, updated_at = ?3 \
             WHERE user_id = ?1 AND auth_session_id != ?2 AND status IN ('pending', 'running')",
            "DELETE FROM job_credentials WHERE job_id IN \
             (SELECT id FROM jobs WHERE user_id = ?1 AND auth_session_id != ?2 AND status = 'cancelled')",
            &[owner.to_string(), current_session_id.to_string()],
        )
        .await
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
}
