use chrono::Utc;
use raw_pipeline::edit_manifest::EditManifest;
use raw_pipeline::edits::Edits;
use uuid::Uuid;
use uuid::fmt::Hyphenated;

use super::*;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub struct EditSnapshot {
    pub id: Uuid,
    pub name: String,
    pub manifest_hash: String,
    pub edits: Edits,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(sqlx::FromRow)]
struct SnapshotRow {
    #[sqlx(try_from = "Hyphenated")]
    id: Uuid,
    name: String,
    edits_json: String,
    created_at: String,
    updated_at: String,
}

impl EditSnapshot {
    fn new(id: Uuid, name: String, edits: Edits, created_at: String, updated_at: String) -> Self {
        let edits = edits.clamped();
        Self {
            id,
            name,
            manifest_hash: edits.stable_hash(),
            edits,
            created_at,
            updated_at,
        }
    }
}

impl TryFrom<SnapshotRow> for EditSnapshot {
    type Error = EditsStoreError;

    fn try_from(row: SnapshotRow) -> Result<Self, Self::Error> {
        let edits = serde_json::from_str(&row.edits_json)?;
        Ok(Self::new(
            row.id,
            row.name,
            edits,
            row.created_at,
            row.updated_at,
        ))
    }
}

impl EditsStore {
    pub async fn list_snapshots(
        &self,
        owner: Uuid,
        asset_id: AssetKey,
    ) -> Result<Vec<EditSnapshot>, EditsStoreError> {
        let rows = sqlx::query_as::<_, SnapshotRow>(
            "SELECT id, name, edits_json, created_at, updated_at FROM edit_snapshots \
             WHERE user_id = ?1 AND asset_id = ?2 ORDER BY created_at DESC, id",
        )
        .bind(owner.to_string())
        .bind(asset_id.to_string())
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .filter_map(|row| {
                let id = row.id;
                EditSnapshot::try_from(row)
                    .inspect_err(|error| {
                        tracing::warn!(snapshot = %id, asset = %asset_id, %error, "skipping unreadable edit snapshot");
                    })
                    .ok()
            })
            .collect())
    }

    pub async fn get_snapshot(
        &self,
        owner: Uuid,
        asset_id: AssetKey,
        id: Uuid,
    ) -> Result<Option<EditSnapshot>, EditsStoreError> {
        sqlx::query_as::<_, SnapshotRow>(
            "SELECT id, name, edits_json, created_at, updated_at FROM edit_snapshots \
             WHERE user_id = ?1 AND asset_id = ?2 AND id = ?3",
        )
        .bind(owner.to_string())
        .bind(asset_id.to_string())
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?
        .map(EditSnapshot::try_from)
        .transpose()
    }

    pub async fn create_snapshot(
        &self,
        owner: Uuid,
        asset_id: AssetKey,
        name: &str,
        manifest: &EditManifest,
    ) -> Result<EditSnapshot, EditsStoreError> {
        let id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();
        let snapshot = EditSnapshot::new(
            id,
            name.to_string(),
            manifest.to_edits(),
            now.clone(),
            now.clone(),
        );
        let edits_json = serde_json::to_string(&snapshot.edits)?;
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        sqlx::query(
            "INSERT INTO edit_snapshots (id, user_id, asset_id, name, edits_json, created_at, \
             updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
        )
        .bind(id.to_string())
        .bind(owner.to_string())
        .bind(asset_id.to_string())
        .bind(name)
        .bind(&edits_json)
        .bind(&now)
        .execute(&mut *tx)
        .await?;
        raster_refs::refresh_raster_refs(&mut tx, owner, asset_id).await?;
        tx.commit().await?;
        Ok(snapshot)
    }

    pub async fn rename_snapshot(
        &self,
        owner: Uuid,
        asset_id: AssetKey,
        id: Uuid,
        name: &str,
    ) -> Result<Option<EditSnapshot>, EditsStoreError> {
        sqlx::query_as::<_, SnapshotRow>(
            "UPDATE edit_snapshots SET name = ?4, updated_at = ?5 \
             WHERE user_id = ?1 AND asset_id = ?2 AND id = ?3 \
             RETURNING id, name, edits_json, created_at, updated_at",
        )
        .bind(owner.to_string())
        .bind(asset_id.to_string())
        .bind(id.to_string())
        .bind(name)
        .bind(Utc::now().to_rfc3339())
        .fetch_optional(&self.pool)
        .await?
        .map(EditSnapshot::try_from)
        .transpose()
    }

    pub async fn delete_snapshot(
        &self,
        owner: Uuid,
        asset_id: AssetKey,
        id: Uuid,
    ) -> Result<bool, EditsStoreError> {
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        let res = sqlx::query(
            "DELETE FROM edit_snapshots WHERE user_id = ?1 AND asset_id = ?2 AND id = ?3",
        )
        .bind(owner.to_string())
        .bind(asset_id.to_string())
        .bind(id.to_string())
        .execute(&mut *tx)
        .await?;
        let deleted = res.rows_affected() > 0;
        if deleted {
            raster_refs::refresh_raster_refs(&mut tx, owner, asset_id).await?;
        }
        tx.commit().await?;
        Ok(deleted)
    }
}
