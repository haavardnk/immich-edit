use chrono::Utc;
use raw_pipeline::edit_manifest::EditManifest;
use sqlx::Row;
use uuid::Uuid;

use super::*;

impl EditsStore {
    pub async fn list_presets(&self, owner: Uuid) -> Result<Vec<PresetRecord>, EditsStoreError> {
        let rows = sqlx::query(
            "SELECT id, name, group_name, manifest_json, created_at, updated_at \
             FROM presets WHERE user_id = ?1 ORDER BY group_name IS NULL, group_name, name",
        )
        .bind(owner.to_string())
        .fetch_all(&self.pool)
        .await?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            out.push(preset_from_row(&row)?);
        }
        Ok(out)
    }

    pub async fn get_preset(
        &self,
        owner: Uuid,
        id: Uuid,
    ) -> Result<Option<PresetRecord>, EditsStoreError> {
        let row = sqlx::query(
            "SELECT id, name, group_name, manifest_json, created_at, updated_at \
             FROM presets WHERE user_id = ?2 AND id = ?1",
        )
        .bind(id.to_string())
        .bind(owner.to_string())
        .fetch_optional(&self.pool)
        .await?;
        let Some(row) = row else {
            return Ok(None);
        };
        Ok(Some(preset_from_row(&row)?))
    }

    pub async fn create_preset(
        &self,
        owner: Uuid,
        name: &str,
        group_name: Option<&str>,
        manifest: &EditManifest,
    ) -> Result<PresetRecord, EditsStoreError> {
        let id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();
        let manifest_json = serde_json::to_string(manifest)?;
        sqlx::query(
            "INSERT INTO presets (id, name, group_name, manifest_json, schema_version, \
             created_at, updated_at, user_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, ?7)",
        )
        .bind(id.to_string())
        .bind(name)
        .bind(group_name)
        .bind(&manifest_json)
        .bind(manifest.schema_version as i64)
        .bind(&now)
        .bind(owner.to_string())
        .execute(&self.pool)
        .await?;
        Ok(PresetRecord {
            id,
            name: name.to_string(),
            group_name: group_name.map(str::to_string),
            manifest: manifest.clone(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub async fn update_preset(
        &self,
        owner: Uuid,
        id: Uuid,
        name: &str,
        group_name: Option<&str>,
        manifest: &EditManifest,
    ) -> Result<Option<PresetRecord>, EditsStoreError> {
        let now = Utc::now().to_rfc3339();
        let manifest_json = serde_json::to_string(manifest)?;
        let res = sqlx::query(
            "UPDATE presets SET name = ?2, group_name = ?3, manifest_json = ?4, \
             schema_version = ?5, updated_at = ?6 WHERE user_id = ?7 AND id = ?1",
        )
        .bind(id.to_string())
        .bind(name)
        .bind(group_name)
        .bind(&manifest_json)
        .bind(manifest.schema_version as i64)
        .bind(&now)
        .bind(owner.to_string())
        .execute(&self.pool)
        .await?;
        if res.rows_affected() == 0 {
            return Ok(None);
        }
        self.get_preset(owner, id).await
    }

    pub async fn delete_preset(&self, owner: Uuid, id: Uuid) -> Result<bool, EditsStoreError> {
        let res = sqlx::query("DELETE FROM presets WHERE user_id = ?2 AND id = ?1")
            .bind(id.to_string())
            .bind(owner.to_string())
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected() > 0)
    }
}

fn preset_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<PresetRecord, EditsStoreError> {
    let id_str: String = row.try_get("id")?;
    let id = Uuid::parse_str(&id_str).map_err(|_| EditsStoreError::Db(sqlx::Error::RowNotFound))?;
    let manifest_json: String = row.try_get("manifest_json")?;
    let manifest: EditManifest = serde_json::from_str(&manifest_json)?;
    Ok(PresetRecord {
        id,
        name: row.try_get("name")?,
        group_name: row.try_get("group_name")?,
        manifest,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
