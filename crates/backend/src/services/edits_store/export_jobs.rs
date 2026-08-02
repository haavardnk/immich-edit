use chrono::Utc;
use sqlx::Row;
use uuid::Uuid;

use super::*;

impl EditsStore {
    pub async fn get_export_job(
        &self,
        owner: Uuid,
        asset_id: Uuid,
        key: &str,
    ) -> Result<Option<ExportJobRecord>, EditsStoreError> {
        let row = sqlx::query(
            "SELECT request_hash, status, immich_asset_id, filename, upload_status, warnings_json \
             FROM export_jobs WHERE user_id = ?3 AND asset_id = ?1 AND idempotency_key = ?2",
        )
        .bind(asset_id.to_string())
        .bind(key)
        .bind(owner.to_string())
        .fetch_optional(&self.pool)
        .await?;
        let Some(row) = row else {
            return Ok(None);
        };
        let warnings_json: String = row.try_get("warnings_json")?;
        let warnings: Vec<String> = serde_json::from_str(&warnings_json).unwrap_or_default();
        let immich_str: Option<String> = row.try_get("immich_asset_id")?;
        let immich_asset_id = immich_str.as_deref().and_then(|s| Uuid::parse_str(s).ok());
        let status_str: String = row.try_get("status")?;
        let status = match status_str.as_str() {
            "pending" => ExportJobStatus::Pending,
            "uploaded" => ExportJobStatus::Uploaded,
            "completed" => ExportJobStatus::Completed,
            _ => ExportJobStatus::Pending,
        };
        Ok(Some(ExportJobRecord {
            request_hash: row.try_get("request_hash")?,
            status,
            immich_asset_id,
            filename: row.try_get("filename")?,
            upload_status: row.try_get("upload_status")?,
            warnings,
        }))
    }

    pub async fn reserve_export_job(
        &self,
        owner: Uuid,
        asset_id: Uuid,
        key: &str,
        request_hash: &str,
    ) -> Result<bool, EditsStoreError> {
        let now = Utc::now().to_rfc3339();
        let result = sqlx::query(
            "INSERT OR IGNORE INTO export_jobs \
             (user_id, asset_id, idempotency_key, request_hash, status, warnings_json, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, 'pending', '[]', ?5, ?5)",
        )
        .bind(owner.to_string())
        .bind(asset_id.to_string())
        .bind(key)
        .bind(request_hash)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn delete_pending_export_job(
        &self,
        owner: Uuid,
        asset_id: Uuid,
        key: &str,
    ) -> Result<(), EditsStoreError> {
        sqlx::query(
            "DELETE FROM export_jobs WHERE user_id = ?1 AND asset_id = ?2 \
             AND idempotency_key = ?3 AND status = 'pending'",
        )
        .bind(owner.to_string())
        .bind(asset_id.to_string())
        .bind(key)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn put_export_job_uploaded(
        &self,
        owner: Uuid,
        asset_id: Uuid,
        key: &str,
        request_hash: &str,
        immich_asset_id: Uuid,
        filename: &str,
        upload_status: &str,
    ) -> Result<(), EditsStoreError> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO export_jobs (user_id, asset_id, idempotency_key, request_hash, status, \
             immich_asset_id, filename, upload_status, warnings_json, created_at, updated_at) \
             VALUES (?8, ?1, ?2, ?3, 'uploaded', ?4, ?5, ?6, '[]', ?7, ?7) \
             ON CONFLICT(user_id, asset_id, idempotency_key) DO UPDATE SET \
               status = excluded.status, \
               immich_asset_id = excluded.immich_asset_id, \
               filename = excluded.filename, \
               upload_status = excluded.upload_status, \
               updated_at = excluded.updated_at",
        )
        .bind(asset_id.to_string())
        .bind(key)
        .bind(request_hash)
        .bind(immich_asset_id.to_string())
        .bind(filename)
        .bind(upload_status)
        .bind(&now)
        .bind(owner.to_string())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn complete_export_job(
        &self,
        owner: Uuid,
        asset_id: Uuid,
        key: &str,
        warnings: &[String],
    ) -> Result<(), EditsStoreError> {
        let now = Utc::now().to_rfc3339();
        let warnings_json = serde_json::to_string(warnings)?;
        sqlx::query(
            "UPDATE export_jobs SET status = 'completed', warnings_json = ?3, updated_at = ?4 \
             WHERE user_id = ?5 AND asset_id = ?1 AND idempotency_key = ?2",
        )
        .bind(asset_id.to_string())
        .bind(key)
        .bind(&warnings_json)
        .bind(&now)
        .bind(owner.to_string())
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
