use chrono::Utc;
use sqlx::{Row, SqlitePool};

#[derive(Debug, thiserror::Error)]
pub enum InstanceStoreError {
    #[error("db: {0}")]
    Db(#[from] sqlx::Error),
    #[error("already configured")]
    AlreadyConfigured,
}

#[derive(Debug, Clone)]
pub struct InstanceConfig {
    pub server_epoch: i64,
    pub immich_url: Option<String>,
    pub configured_at: Option<String>,
}

impl InstanceConfig {
    pub fn is_configured(&self) -> bool {
        self.server_epoch > 0
    }
}

#[derive(Debug, Clone)]
pub struct InstanceStore {
    pool: SqlitePool,
}

impl InstanceStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get(&self) -> Result<InstanceConfig, InstanceStoreError> {
        let row = sqlx::query(
            "SELECT server_epoch, immich_url, configured_at FROM instance_config WHERE id = 1",
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(InstanceConfig {
            server_epoch: row.get("server_epoch"),
            immich_url: row.get("immich_url"),
            configured_at: row.get("configured_at"),
        })
    }

    pub async fn claim(&self, immich_url: &str) -> Result<i64, InstanceStoreError> {
        let now = Utc::now().to_rfc3339();
        let res = sqlx::query(
            "UPDATE instance_config SET server_epoch = 1, immich_url = ?1, configured_at = ?2 \
             WHERE id = 1 AND server_epoch = 0",
        )
        .bind(immich_url)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        if res.rows_affected() == 0 {
            return Err(InstanceStoreError::AlreadyConfigured);
        }
        Ok(1)
    }

    pub async fn rebind(&self, immich_url: &str) -> Result<i64, InstanceStoreError> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE instance_config SET server_epoch = server_epoch + 1, immich_url = ?1, \
             configured_at = ?2 WHERE id = 1",
        )
        .bind(immich_url)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        Ok(self.get().await?.server_epoch)
    }

    pub async fn has_encrypted_secrets(&self) -> Result<bool, InstanceStoreError> {
        let row = sqlx::query("SELECT EXISTS(SELECT 1 FROM sessions) AS present")
            .fetch_one(&self.pool)
            .await?;
        let present: i64 = row.get("present");
        Ok(present != 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::edits_store::EditsStore;

    async fn store() -> InstanceStore {
        let edits = EditsStore::migrated_memory().await.unwrap();
        InstanceStore::new(edits.pool())
    }

    #[tokio::test]
    async fn starts_unconfigured() {
        let s = store().await;
        let cfg = s.get().await.unwrap();
        assert!(!cfg.is_configured());
        assert_eq!(cfg.server_epoch, 0);
        assert!(cfg.immich_url.is_none());
    }

    #[tokio::test]
    async fn first_claim_wins_second_rejected() {
        let s = store().await;
        let epoch = s.claim("https://immich.example").await.unwrap();
        assert_eq!(epoch, 1);
        let cfg = s.get().await.unwrap();
        assert!(cfg.is_configured());
        assert_eq!(cfg.immich_url.as_deref(), Some("https://immich.example"));
        let second = s.claim("https://evil.example").await;
        assert!(matches!(second, Err(InstanceStoreError::AlreadyConfigured)));
        assert_eq!(
            s.get().await.unwrap().immich_url.as_deref(),
            Some("https://immich.example")
        );
    }

    #[tokio::test]
    async fn rebind_rotates_epoch() {
        let s = store().await;
        s.claim("https://a.example").await.unwrap();
        let epoch = s.rebind("https://b.example").await.unwrap();
        assert_eq!(epoch, 2);
        assert_eq!(
            s.get().await.unwrap().immich_url.as_deref(),
            Some("https://b.example")
        );
    }
}
