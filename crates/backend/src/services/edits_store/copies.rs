use std::collections::HashMap;

use chrono::Utc;
use sqlx::Row;
use uuid::Uuid;

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyRecord {
    pub id: AssetKey,
    pub source_asset_id: Uuid,
    pub name: Option<String>,
    pub created_at: String,
}

impl EditsStore {
    pub async fn create_copy(
        &self,
        owner: Uuid,
        source: Uuid,
        name: Option<&str>,
    ) -> Result<CopyRecord, EditsStoreError> {
        let now = Utc::now().to_rfc3339();
        let source_str = source.to_string();
        let row = sqlx::query(
            "INSERT INTO asset_copies (user_id, id, source_asset_id, idx, name, created_at) \
             SELECT ?1, printf('%s_%d', ?2, n.idx), ?2, n.idx, ?3, ?4 FROM \
             (SELECT COALESCE(MAX(idx), 0) + 1 AS idx FROM asset_copies \
              WHERE user_id = ?1 AND source_asset_id = ?2) n \
             RETURNING id, source_asset_id, name, created_at",
        )
        .bind(owner.to_string())
        .bind(&source_str)
        .bind(name)
        .bind(&now)
        .fetch_one(&self.pool)
        .await?;
        copy_from_row(&row)
    }

    pub async fn list_copies(
        &self,
        owner: Uuid,
        source: Uuid,
    ) -> Result<Vec<CopyRecord>, EditsStoreError> {
        let rows = sqlx::query(
            "SELECT id, source_asset_id, name, created_at FROM asset_copies \
             WHERE user_id = ?1 AND source_asset_id = ?2 AND deleted = 0 ORDER BY idx",
        )
        .bind(owner.to_string())
        .bind(source.to_string())
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(copy_from_row).collect()
    }

    pub async fn expand_copies(
        &self,
        owner: Uuid,
        sources: &[Uuid],
    ) -> Result<HashMap<Uuid, Vec<CopyRecord>>, EditsStoreError> {
        let mut out: HashMap<Uuid, Vec<CopyRecord>> = HashMap::new();
        if sources.is_empty() {
            return Ok(out);
        }
        let ids: Vec<String> = sources.iter().map(Uuid::to_string).collect();
        let rows = sqlx::query(
            "SELECT id, source_asset_id, name, created_at FROM asset_copies \
             WHERE user_id = ?1 AND deleted = 0 \
             AND source_asset_id IN (SELECT value FROM json_each(?2)) ORDER BY idx",
        )
        .bind(owner.to_string())
        .bind(serde_json::to_string(&ids)?)
        .fetch_all(&self.pool)
        .await?;
        for row in rows {
            let record = copy_from_row(&row)?;
            out.entry(record.source_asset_id).or_default().push(record);
        }
        Ok(out)
    }

    pub async fn rename_copy(
        &self,
        owner: Uuid,
        id: AssetKey,
        name: Option<&str>,
    ) -> Result<Option<CopyRecord>, EditsStoreError> {
        let row = sqlx::query(
            "UPDATE asset_copies SET name = ?3 WHERE user_id = ?1 AND id = ?2 AND deleted = 0 \
             RETURNING id, source_asset_id, name, created_at",
        )
        .bind(owner.to_string())
        .bind(id.to_string())
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;
        let Some(row) = row else {
            return Ok(None);
        };
        Ok(Some(copy_from_row(&row)?))
    }

    pub async fn delete_copy(&self, owner: Uuid, id: AssetKey) -> Result<bool, EditsStoreError> {
        let owner_str = owner.to_string();
        let id_str = id.to_string();
        let res = sqlx::query(
            "UPDATE asset_copies SET deleted = 1, name = NULL \
             WHERE user_id = ?1 AND id = ?2 AND deleted = 0",
        )
        .bind(&owner_str)
        .bind(&id_str)
        .execute(&self.pool)
        .await?;
        if res.rows_affected() == 0 {
            return Ok(false);
        }
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM edits WHERE user_id = ?1 AND asset_id = ?2")
            .bind(&owner_str)
            .bind(&id_str)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM edits_history WHERE user_id = ?1 AND asset_id = ?2")
            .bind(&owner_str)
            .bind(&id_str)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM raster_refs WHERE user_id = ?1 AND asset_id = ?2")
            .bind(&owner_str)
            .bind(&id_str)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM export_jobs WHERE user_id = ?1 AND asset_id = ?2")
            .bind(&owner_str)
            .bind(&id_str)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(true)
    }
}

fn copy_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<CopyRecord, EditsStoreError> {
    let id: String = row.try_get("id")?;
    let source_asset_id: String = row.try_get("source_asset_id")?;
    Ok(CopyRecord {
        id: id.parse().map_err(|_| sqlx::Error::RowNotFound)?,
        source_asset_id: source_asset_id
            .parse()
            .map_err(|_| sqlx::Error::RowNotFound)?,
        name: row.try_get("name")?,
        created_at: row.try_get("created_at")?,
    })
}
