use std::collections::HashMap;

use chrono::Utc;
use uuid::Uuid;

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CopyRecord {
    #[sqlx(try_from = "String")]
    pub id: AssetKey,
    #[sqlx(try_from = "Hyphenated")]
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
        Ok(sqlx::query_as::<_, CopyRecord>(
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
        .await?)
    }

    pub async fn list_copies(
        &self,
        owner: Uuid,
        source: Uuid,
    ) -> Result<Vec<CopyRecord>, EditsStoreError> {
        Ok(sqlx::query_as::<_, CopyRecord>(
            "SELECT id, source_asset_id, name, created_at FROM asset_copies \
             WHERE user_id = ?1 AND source_asset_id = ?2 AND deleted = 0 ORDER BY idx",
        )
        .bind(owner.to_string())
        .bind(source.to_string())
        .fetch_all(&self.pool)
        .await?)
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
        let records = sqlx::query_as::<_, CopyRecord>(
            "SELECT id, source_asset_id, name, created_at FROM asset_copies \
             WHERE user_id = ?1 AND deleted = 0 \
             AND source_asset_id IN (SELECT value FROM json_each(?2)) ORDER BY idx",
        )
        .bind(owner.to_string())
        .bind(serde_json::to_string(&ids)?)
        .fetch_all(&self.pool)
        .await?;
        for record in records {
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
        Ok(sqlx::query_as::<_, CopyRecord>(
            "UPDATE asset_copies SET name = ?3 WHERE user_id = ?1 AND id = ?2 AND deleted = 0 \
             RETURNING id, source_asset_id, name, created_at",
        )
        .bind(owner.to_string())
        .bind(id.to_string())
        .bind(name)
        .fetch_optional(&self.pool)
        .await?)
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
        sqlx::query("DELETE FROM edit_snapshots WHERE user_id = ?1 AND asset_id = ?2")
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
        sqlx::query("DELETE FROM export_outputs WHERE user_id = ?1 AND asset_id = ?2")
            .bind(&owner_str)
            .bind(&id_str)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(true)
    }

    pub async fn has_edits_for_source(
        &self,
        owner: Uuid,
        source: Uuid,
    ) -> Result<bool, EditsStoreError> {
        let source = source.to_string();
        Ok(sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM edits WHERE user_id = ?1 \
             AND (asset_id = ?2 OR substr(asset_id, 1, length(?2) + 1) = ?2 || '_')) \
             OR EXISTS (SELECT 1 FROM asset_copies WHERE user_id = ?1 \
             AND source_asset_id = ?2 AND deleted = 0)",
        )
        .bind(owner.to_string())
        .bind(&source)
        .fetch_one(&self.pool)
        .await?)
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{O, store};
    use super::*;

    #[tokio::test]
    async fn edits_of_a_source_include_its_live_copies() {
        let s = store().await;
        let [master, copied, bare, deleted, untouched] = [(); 5].map(|_| Uuid::new_v4());
        let copy = s.create_copy(O, copied, None).await.unwrap().id;
        s.create_copy(O, bare, None).await.unwrap();
        let gone = s.create_copy(O, deleted, None).await.unwrap().id;
        assert!(s.delete_copy(O, gone).await.unwrap());
        s.put(O, AssetKey::master(master), EditWrite::default())
            .await
            .unwrap();
        s.put(O, copy, EditWrite::default()).await.unwrap();

        let mut found = Vec::new();
        for source in [master, copied, bare, deleted, untouched] {
            found.push(s.has_edits_for_source(O, source).await.unwrap());
        }
        found.push(s.has_edits_for_source(Uuid::new_v4(), bare).await.unwrap());

        assert_eq!(found, [true, true, true, false, false, false]);
    }
}
