use chrono::Utc;
use serde::Serialize;
use uuid::Uuid;

use super::*;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct SavedViewRecord {
    #[sqlx(try_from = "Hyphenated")]
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub created_at: String,
    pub updated_at: String,
}

impl EditsStore {
    pub async fn list_saved_views(
        &self,
        owner: Uuid,
    ) -> Result<Vec<SavedViewRecord>, EditsStoreError> {
        Ok(sqlx::query_as::<_, SavedViewRecord>(
            "SELECT id, name, url, created_at, updated_at \
             FROM saved_views WHERE user_id = ?1 ORDER BY name COLLATE NOCASE, created_at",
        )
        .bind(owner.to_string())
        .fetch_all(&self.pool)
        .await?)
    }

    pub async fn get_saved_view(
        &self,
        owner: Uuid,
        id: Uuid,
    ) -> Result<Option<SavedViewRecord>, EditsStoreError> {
        Ok(sqlx::query_as::<_, SavedViewRecord>(
            "SELECT id, name, url, created_at, updated_at \
             FROM saved_views WHERE user_id = ?2 AND id = ?1",
        )
        .bind(id.to_string())
        .bind(owner.to_string())
        .fetch_optional(&self.pool)
        .await?)
    }

    pub async fn create_saved_view(
        &self,
        owner: Uuid,
        name: &str,
        url: &str,
    ) -> Result<SavedViewRecord, EditsStoreError> {
        let id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO saved_views (id, user_id, name, url, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
        )
        .bind(id.to_string())
        .bind(owner.to_string())
        .bind(name)
        .bind(url)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        Ok(SavedViewRecord {
            id,
            name: name.to_string(),
            url: url.to_string(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub async fn update_saved_view(
        &self,
        owner: Uuid,
        id: Uuid,
        name: &str,
        url: &str,
    ) -> Result<Option<SavedViewRecord>, EditsStoreError> {
        let res = sqlx::query(
            "UPDATE saved_views SET name = ?2, url = ?3, updated_at = ?4 \
             WHERE user_id = ?5 AND id = ?1",
        )
        .bind(id.to_string())
        .bind(name)
        .bind(url)
        .bind(Utc::now().to_rfc3339())
        .bind(owner.to_string())
        .execute(&self.pool)
        .await?;
        if res.rows_affected() == 0 {
            return Ok(None);
        }
        self.get_saved_view(owner, id).await
    }

    pub async fn delete_saved_view(&self, owner: Uuid, id: Uuid) -> Result<bool, EditsStoreError> {
        let res = sqlx::query("DELETE FROM saved_views WHERE user_id = ?2 AND id = ?1")
            .bind(id.to_string())
            .bind(owner.to_string())
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected() > 0)
    }
}
