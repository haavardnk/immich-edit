use chrono::Utc;
use sqlx::Row;

use super::*;

impl AuthStore {
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
}
