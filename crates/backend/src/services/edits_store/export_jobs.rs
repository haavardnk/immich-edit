use chrono::Utc;
use sqlx::Row;
use sqlx::types::Json;
use uuid::Uuid;

use super::export_outputs::{ExportOutput, upsert_export_output};
use super::*;

#[derive(Debug, Clone, Copy)]
pub struct ExportJobCompletion<'a> {
    pub warnings: &'a [String],
    pub original_trashed: bool,
    pub replaced_asset_id: Option<Uuid>,
    pub output: Option<ExportOutput<'a>>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ExportJobBaseline {
    pub previous_asset_id: Option<Uuid>,
    pub replace_asset_id: Option<Uuid>,
}

impl EditsStore {
    pub async fn get_export_job(
        &self,
        job: ExportJobKey<'_>,
    ) -> Result<Option<ExportJobRecord>, EditsStoreError> {
        let row = sqlx::query(
            "SELECT request_hash, status, immich_asset_id, filename, upload_status, warnings_json, \
             original_trashed, replaced_asset_id, previous_asset_id, replace_asset_id \
             FROM export_jobs WHERE user_id = ?3 AND asset_id = ?1 AND idempotency_key = ?2",
        )
        .bind(job.asset_id.to_string())
        .bind(job.key)
        .bind(job.owner.to_string())
        .fetch_optional(&self.pool)
        .await?;
        let Some(row) = row else {
            return Ok(None);
        };
        Ok(Some(ExportJobRecord {
            request_hash: row.try_get("request_hash")?,
            status: row.try_get("status")?,
            immich_asset_id: row
                .try_get::<Option<Hyphenated>, _>("immich_asset_id")?
                .map(Uuid::from),
            filename: row.try_get("filename")?,
            upload_status: row.try_get("upload_status")?,
            warnings: row.try_get::<Json<Vec<String>>, _>("warnings_json")?.0,
            original_trashed: row.try_get("original_trashed")?,
            replaced_asset_id: row
                .try_get::<Option<Hyphenated>, _>("replaced_asset_id")?
                .map(Uuid::from),
            previous_asset_id: row
                .try_get::<Option<Hyphenated>, _>("previous_asset_id")?
                .map(Uuid::from),
            replace_asset_id: row
                .try_get::<Option<Hyphenated>, _>("replace_asset_id")?
                .map(Uuid::from),
        }))
    }

    pub async fn reserve_export_job(
        &self,
        job: ExportJobKey<'_>,
        request_hash: &str,
        baseline: ExportJobBaseline,
    ) -> Result<bool, EditsStoreError> {
        let now = Utc::now().to_rfc3339();
        let result = sqlx::query(
            "INSERT OR IGNORE INTO export_jobs \
             (user_id, asset_id, idempotency_key, request_hash, status, warnings_json, created_at, \
             updated_at, previous_asset_id, replace_asset_id) \
             VALUES (?1, ?2, ?3, ?4, 'pending', '[]', ?5, ?5, ?6, ?7)",
        )
        .bind(job.owner.to_string())
        .bind(job.asset_id.to_string())
        .bind(job.key)
        .bind(request_hash)
        .bind(&now)
        .bind(baseline.previous_asset_id.map(|id| id.to_string()))
        .bind(baseline.replace_asset_id.map(|id| id.to_string()))
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn delete_pending_export_job(
        &self,
        job: ExportJobKey<'_>,
    ) -> Result<(), EditsStoreError> {
        sqlx::query(
            "DELETE FROM export_jobs WHERE user_id = ?1 AND asset_id = ?2 \
             AND idempotency_key = ?3 AND status = 'pending'",
        )
        .bind(job.owner.to_string())
        .bind(job.asset_id.to_string())
        .bind(job.key)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn put_export_job_uploaded(
        &self,
        job: ExportJobKey<'_>,
        request_hash: &str,
        immich_asset_id: Uuid,
        filename: &str,
        upload_status: &str,
        warnings: &[String],
    ) -> Result<(), EditsStoreError> {
        let now = Utc::now().to_rfc3339();
        let warnings_json = serde_json::to_string(warnings)?;
        sqlx::query(
            "INSERT INTO export_jobs (user_id, asset_id, idempotency_key, request_hash, status, \
             immich_asset_id, filename, upload_status, warnings_json, created_at, updated_at) \
             VALUES (?8, ?1, ?2, ?3, 'uploaded', ?4, ?5, ?6, ?9, ?7, ?7) \
             ON CONFLICT(user_id, asset_id, idempotency_key) DO UPDATE SET \
               status = excluded.status, \
               immich_asset_id = excluded.immich_asset_id, \
               filename = excluded.filename, \
               upload_status = excluded.upload_status, \
               warnings_json = excluded.warnings_json, \
               updated_at = excluded.updated_at",
        )
        .bind(job.asset_id.to_string())
        .bind(job.key)
        .bind(request_hash)
        .bind(immich_asset_id.to_string())
        .bind(filename)
        .bind(upload_status)
        .bind(&now)
        .bind(job.owner.to_string())
        .bind(&warnings_json)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn complete_export_job(
        &self,
        job: ExportJobKey<'_>,
        completion: ExportJobCompletion<'_>,
    ) -> Result<bool, EditsStoreError> {
        let now = Utc::now().to_rfc3339();
        let warnings_json = serde_json::to_string(completion.warnings)?;
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "UPDATE export_jobs SET status = 'completed', warnings_json = ?3, updated_at = ?4, \
             original_trashed = ?6, replaced_asset_id = ?7 \
             WHERE user_id = ?5 AND asset_id = ?1 AND idempotency_key = ?2",
        )
        .bind(job.asset_id.to_string())
        .bind(job.key)
        .bind(&warnings_json)
        .bind(&now)
        .bind(job.owner.to_string())
        .bind(completion.original_trashed)
        .bind(completion.replaced_asset_id.map(|id| id.to_string()))
        .execute(&mut *tx)
        .await?;
        let written = match completion.output {
            Some(output) => upsert_export_output(&mut *tx, output).await?,
            None => false,
        };
        tx.commit().await?;
        Ok(written)
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{O, key, store};
    use super::*;

    fn job(key: AssetKey) -> ExportJobKey<'static> {
        ExportJobKey {
            owner: O,
            asset_id: key,
            key: "k1",
        }
    }

    #[tokio::test]
    async fn a_reserved_job_keeps_its_baseline_through_upload() {
        let s = store().await;
        let job = job(key());
        let (previous, replace) = (Uuid::new_v4(), Uuid::new_v4());
        let baseline = ExportJobBaseline {
            previous_asset_id: Some(previous),
            replace_asset_id: Some(replace),
        };

        assert!(s.reserve_export_job(job, "h1", baseline).await.unwrap());
        s.put_export_job_uploaded(job, "h1", Uuid::new_v4(), "f.jpg", "created", &[])
            .await
            .unwrap();

        let record = s.get_export_job(job).await.unwrap().unwrap();
        assert_eq!(record.status, ExportJobStatus::Uploaded);
        assert_eq!(record.previous_asset_id, Some(previous));
        assert_eq!(record.replace_asset_id, Some(replace));
    }

    #[tokio::test]
    async fn completion_records_the_replace_and_the_output_together() {
        let s = store().await;
        let asset = key();
        let job = job(asset);
        let (previous, upload) = (Uuid::new_v4(), Uuid::new_v4());
        let output = ExportOutputKey {
            owner: O,
            asset_id: asset,
            preset: "web",
        };
        s.put_export_output(output.output(previous, None))
            .await
            .unwrap();
        s.put_export_job_uploaded(job, "h1", upload, "f.jpg", "created", &[])
            .await
            .unwrap();

        let completion = ExportJobCompletion {
            warnings: &["w1".into()],
            original_trashed: false,
            replaced_asset_id: Some(previous),
            output: Some(output.output(upload, Some(previous))),
        };
        assert!(s.complete_export_job(job, completion).await.unwrap());

        let record = s.get_export_job(job).await.unwrap().unwrap();
        assert_eq!(record.status, ExportJobStatus::Completed);
        assert_eq!(record.replaced_asset_id, Some(previous));
        assert_eq!(s.get_export_output(output).await.unwrap(), Some(upload));
    }

    #[tokio::test]
    async fn completion_leaves_an_output_a_newer_export_recorded() {
        let s = store().await;
        let asset = key();
        let job = job(asset);
        let (previous, newer, upload) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        let output = ExportOutputKey {
            owner: O,
            asset_id: asset,
            preset: "",
        };
        s.put_export_output(output.output(newer, None))
            .await
            .unwrap();
        s.put_export_job_uploaded(job, "h1", upload, "f.jpg", "created", &[])
            .await
            .unwrap();

        let completion = ExportJobCompletion {
            warnings: &[],
            original_trashed: false,
            replaced_asset_id: None,
            output: Some(output.output(upload, Some(previous))),
        };
        assert!(!s.complete_export_job(job, completion).await.unwrap());

        let record = s.get_export_job(job).await.unwrap().unwrap();
        assert_eq!(record.status, ExportJobStatus::Completed);
        assert_eq!(s.get_export_output(output).await.unwrap(), Some(newer));
    }
}
