use uuid::Uuid;

use super::*;

impl EditsStore {
    pub async fn purge_owner(&self, owner: Uuid) -> Result<(), EditsStoreError> {
        let owner_str = owner.to_string();
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM edits WHERE user_id = ?1")
            .bind(&owner_str)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM edits_history WHERE user_id = ?1")
            .bind(&owner_str)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM presets WHERE user_id = ?1")
            .bind(&owner_str)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM export_jobs WHERE user_id = ?1")
            .bind(&owner_str)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM raster_refs WHERE user_id = ?1")
            .bind(&owner_str)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn purge_all(&self) -> Result<(), EditsStoreError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM edits").execute(&mut *tx).await?;
        sqlx::query("DELETE FROM edits_history")
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM presets").execute(&mut *tx).await?;
        sqlx::query("DELETE FROM export_jobs")
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM raster_refs")
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }
}
