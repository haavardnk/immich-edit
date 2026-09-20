use sqlx::SqliteConnection;
use uuid::Uuid;

use super::*;

impl JobStore {
    pub(super) async fn insert_credential(
        &self,
        conn: &mut SqliteConnection,
        job_id: Uuid,
        cred: &[u8],
        auth_kind: AuthKind,
    ) -> Result<(), JobStoreError> {
        let enc = self.crypto.encrypt(cred)?;
        sqlx::query(
            "INSERT INTO job_credentials (job_id, ciphertext, nonce, key_version, auth_kind) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .bind(job_id.to_string())
        .bind(enc.ciphertext)
        .bind(enc.nonce)
        .bind(enc.key_version)
        .bind(auth_kind.as_str())
        .execute(conn)
        .await?;
        Ok(())
    }

    pub async fn job_credential(
        &self,
        job_id: Uuid,
    ) -> Result<Option<(SecretBytes, AuthKind)>, JobStoreError> {
        let row = sqlx::query(
            "SELECT ciphertext, nonce, key_version, auth_kind FROM job_credentials WHERE job_id = ?1",
        )
        .bind(job_id.to_string())
        .fetch_optional(&self.pool)
        .await?;
        let Some(row) = row else {
            return Ok(None);
        };
        let enc = crate::services::crypto::Encrypted {
            ciphertext: row.get::<Vec<u8>, _>("ciphertext"),
            nonce: row.get::<Vec<u8>, _>("nonce"),
            key_version: row.get::<i64, _>("key_version"),
        };
        let cred = self.crypto.decrypt(&enc)?;
        let auth_kind = AuthKind::from_wire(&row.get::<String, _>("auth_kind"));
        Ok(Some((cred, auth_kind)))
    }
}

pub(super) async fn delete_credential(
    conn: &mut SqliteConnection,
    job_id: Uuid,
) -> Result<(), JobStoreError> {
    sqlx::query("DELETE FROM job_credentials WHERE job_id = ?1")
        .bind(job_id.to_string())
        .execute(conn)
        .await?;
    Ok(())
}
