use chrono::Utc;
use serde::Serialize;
use uuid::Uuid;

use super::*;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ExportPresetRecord {
    #[sqlx(try_from = "Hyphenated")]
    pub id: Uuid,
    pub name: String,
    #[sqlx(rename = "form_json", json)]
    pub form: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

impl EditsStore {
    pub async fn list_export_presets(
        &self,
        owner: Uuid,
    ) -> Result<Vec<ExportPresetRecord>, EditsStoreError> {
        Ok(sqlx::query_as::<_, ExportPresetRecord>(
            "SELECT id, name, form_json, created_at, updated_at \
             FROM export_presets WHERE user_id = ?1 ORDER BY name COLLATE NOCASE, created_at",
        )
        .bind(owner.to_string())
        .fetch_all(&self.pool)
        .await?)
    }

    pub async fn get_export_preset(
        &self,
        owner: Uuid,
        id: Uuid,
    ) -> Result<Option<ExportPresetRecord>, EditsStoreError> {
        Ok(sqlx::query_as::<_, ExportPresetRecord>(
            "SELECT id, name, form_json, created_at, updated_at \
             FROM export_presets WHERE user_id = ?2 AND id = ?1",
        )
        .bind(id.to_string())
        .bind(owner.to_string())
        .fetch_optional(&self.pool)
        .await?)
    }

    pub async fn create_export_preset(
        &self,
        owner: Uuid,
        name: &str,
        form: &serde_json::Value,
    ) -> Result<ExportPresetRecord, EditsStoreError> {
        let id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO export_presets (id, user_id, name, form_json, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
        )
        .bind(id.to_string())
        .bind(owner.to_string())
        .bind(name)
        .bind(serde_json::to_string(form)?)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        Ok(ExportPresetRecord {
            id,
            name: name.to_string(),
            form: form.clone(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub async fn update_export_preset(
        &self,
        owner: Uuid,
        id: Uuid,
        name: &str,
        form: &serde_json::Value,
    ) -> Result<Option<ExportPresetRecord>, EditsStoreError> {
        let res = sqlx::query(
            "UPDATE export_presets SET name = ?2, form_json = ?3, updated_at = ?4 \
             WHERE user_id = ?5 AND id = ?1",
        )
        .bind(id.to_string())
        .bind(name)
        .bind(serde_json::to_string(form)?)
        .bind(Utc::now().to_rfc3339())
        .bind(owner.to_string())
        .execute(&self.pool)
        .await?;
        if res.rows_affected() == 0 {
            return Ok(None);
        }
        self.get_export_preset(owner, id).await
    }

    pub async fn delete_export_preset(
        &self,
        owner: Uuid,
        id: Uuid,
    ) -> Result<bool, EditsStoreError> {
        let res = sqlx::query("DELETE FROM export_presets WHERE user_id = ?2 AND id = ?1")
            .bind(id.to_string())
            .bind(owner.to_string())
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected() > 0)
    }
}
