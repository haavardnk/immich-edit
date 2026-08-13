use raw_pipeline::edits::Edits;
use sqlx::Row;
use uuid::Uuid;

use super::*;

impl EditsStore {
    pub async fn rebuild_raster_refs(&self) -> Result<usize, EditsStoreError> {
        let rows = sqlx::query(
            "SELECT user_id, asset_id, edits_json FROM edits \
             UNION ALL \
             SELECT user_id, asset_id, edits_json FROM edits_history WHERE edits_json IS NOT NULL",
        )
        .fetch_all(&self.pool)
        .await?;
        let mut triples: Vec<(String, String, String)> = Vec::new();
        for row in rows {
            let user_id: String = row.try_get("user_id")?;
            let asset_id: String = row.try_get("asset_id")?;
            let edits_json: String = row.try_get("edits_json")?;
            let Ok(edits) = serde_json::from_str::<Edits>(&edits_json) else {
                continue;
            };
            for id in edits.retained_raster_ids() {
                let triple = (user_id.clone(), asset_id.clone(), id);
                if !triples.contains(&triple) {
                    triples.push(triple);
                }
            }
        }
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM raster_refs")
            .execute(&mut *tx)
            .await?;
        for (user_id, asset_id, raster_id) in &triples {
            sqlx::query(
                "INSERT OR IGNORE INTO raster_refs (user_id, asset_id, raster_id) \
                 VALUES (?1, ?2, ?3)",
            )
            .bind(user_id)
            .bind(asset_id)
            .bind(raster_id)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(triples.len())
    }

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
