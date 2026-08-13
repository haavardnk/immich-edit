use uuid::Uuid;

use super::*;

pub fn parse_uuid(s: String) -> Result<Uuid, JobStoreError> {
    Uuid::parse_str(&s).map_err(|_| JobStoreError::Corrupt(format!("uuid {s}")))
}

pub fn job_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<JobRecord, JobStoreError> {
    Ok(JobRecord {
        id: parse_uuid(row.get("id"))?,
        user_id: parse_uuid(row.get("user_id"))?,
        server_epoch: row.get("server_epoch"),
        auth_session_id: row
            .get::<Option<String>, _>("auth_session_id")
            .map(parse_uuid)
            .transpose()?,
        kind: row.get("kind"),
        status: JobStatus::from_str(&row.get::<String, _>("status"))?,
        target: serde_json::from_str(&row.get::<String, _>("target_json"))?,
        params: serde_json::from_str(&row.get::<String, _>("params_json"))?,
        total: row.get("total"),
        completed: row.get("completed"),
        failed: row.get("failed"),
        cancelled_at: row.get("cancelled_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

pub fn item_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<JobItemRecord, JobStoreError> {
    let result = match row.get::<Option<String>, _>("result_json") {
        Some(s) => Some(serde_json::from_str(&s)?),
        None => None,
    };
    Ok(JobItemRecord {
        id: parse_uuid(row.get("id"))?,
        job_id: parse_uuid(row.get("job_id"))?,
        asset_id: row.get("asset_id"),
        status: JobItemStatus::from_str(&row.get::<String, _>("status"))?,
        error: row.get("error"),
        result,
        idempotency_key: row.get("idempotency_key"),
        attempts: row.get("attempts"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}
