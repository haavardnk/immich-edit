use chrono::Utc;
use uuid::Uuid;

use super::*;

#[derive(Debug, Clone, Copy)]
pub struct ExportOutputKey<'a> {
    pub owner: Uuid,
    pub asset_id: AssetKey,
    pub preset: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub struct ExportOutput<'a> {
    pub key: ExportOutputKey<'a>,
    pub immich_asset_id: Uuid,
    pub previous: Option<Uuid>,
}

impl<'a> ExportOutputKey<'a> {
    pub fn output(self, immich_asset_id: Uuid, previous: Option<Uuid>) -> ExportOutput<'a> {
        ExportOutput {
            key: self,
            immich_asset_id,
            previous,
        }
    }
}

pub(super) async fn upsert_export_output<'e, E: SqliteExecutor<'e>>(
    exec: E,
    output: ExportOutput<'_>,
) -> Result<bool, EditsStoreError> {
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query(
        "INSERT INTO export_outputs (user_id, asset_id, preset, immich_asset_id, created_at, \
         updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?5) \
         ON CONFLICT(user_id, asset_id, preset) DO UPDATE SET \
           immich_asset_id = excluded.immich_asset_id, \
           updated_at = excluded.updated_at \
         WHERE export_outputs.immich_asset_id IN (?6, excluded.immich_asset_id)",
    )
    .bind(output.key.owner.to_string())
    .bind(output.key.asset_id.to_string())
    .bind(output.key.preset)
    .bind(output.immich_asset_id.to_string())
    .bind(&now)
    .bind(output.previous.map(|id| id.to_string()))
    .execute(exec)
    .await?;
    Ok(result.rows_affected() > 0)
}

impl EditsStore {
    pub async fn get_export_output(
        &self,
        key: ExportOutputKey<'_>,
    ) -> Result<Option<Uuid>, EditsStoreError> {
        let id: Option<Hyphenated> = sqlx::query_scalar(
            "SELECT immich_asset_id FROM export_outputs \
             WHERE user_id = ?1 AND asset_id = ?2 AND preset = ?3",
        )
        .bind(key.owner.to_string())
        .bind(key.asset_id.to_string())
        .bind(key.preset)
        .fetch_optional(&self.pool)
        .await?;
        Ok(id.map(Uuid::from))
    }

    pub async fn put_export_output(
        &self,
        output: ExportOutput<'_>,
    ) -> Result<bool, EditsStoreError> {
        upsert_export_output(&self.pool, output).await
    }

    pub async fn export_output_shared(
        &self,
        key: ExportOutputKey<'_>,
        immich_asset_id: Uuid,
    ) -> Result<bool, EditsStoreError> {
        Ok(sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM export_outputs \
             WHERE user_id = ?1 AND immich_asset_id = ?4 \
             AND NOT (asset_id = ?2 AND preset = ?3))",
        )
        .bind(key.owner.to_string())
        .bind(key.asset_id.to_string())
        .bind(key.preset)
        .bind(immich_asset_id.to_string())
        .fetch_one(&self.pool)
        .await?)
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{O, key, store};
    use super::*;

    fn output(asset_id: AssetKey, preset: &str) -> ExportOutputKey<'_> {
        ExportOutputKey {
            owner: O,
            asset_id,
            preset,
        }
    }

    #[tokio::test]
    async fn an_output_is_recorded_only_over_the_export_it_replaced() {
        let s = store().await;
        let web = output(key(), "web");
        let (first, second, third) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());

        let recorded = [
            s.put_export_output(web.output(first, None)).await.unwrap(),
            s.put_export_output(web.output(second, None)).await.unwrap(),
            s.put_export_output(web.output(second, Some(third)))
                .await
                .unwrap(),
            s.put_export_output(web.output(second, Some(first)))
                .await
                .unwrap(),
            s.put_export_output(web.output(second, None)).await.unwrap(),
        ];

        assert_eq!(recorded, [true, false, false, true, true]);
        assert_eq!(s.get_export_output(web).await.unwrap(), Some(second));
    }

    #[tokio::test]
    async fn outputs_are_kept_apart_per_preset_key() {
        let s = store().await;
        let asset = key();
        let (web, blank) = (Uuid::new_v4(), Uuid::new_v4());

        s.put_export_output(output(asset, "web").output(web, None))
            .await
            .unwrap();
        s.put_export_output(output(asset, "").output(blank, None))
            .await
            .unwrap();

        assert_eq!(
            s.get_export_output(output(asset, "web")).await.unwrap(),
            Some(web)
        );
        assert_eq!(
            s.get_export_output(output(asset, "")).await.unwrap(),
            Some(blank)
        );
        assert_eq!(
            s.get_export_output(output(asset, "Web")).await.unwrap(),
            None
        );
    }

    #[tokio::test]
    async fn an_output_is_shared_while_another_key_records_it() {
        let s = store().await;
        let asset = key();
        let (web, print) = (output(asset, "web"), output(asset, "print"));
        let (shared, other) = (Uuid::new_v4(), Uuid::new_v4());
        s.put_export_output(web.output(shared, None)).await.unwrap();

        assert!(!s.export_output_shared(web, shared).await.unwrap());
        s.put_export_output(print.output(shared, None))
            .await
            .unwrap();
        assert!(s.export_output_shared(web, shared).await.unwrap());
        s.put_export_output(print.output(other, Some(shared)))
            .await
            .unwrap();
        assert!(!s.export_output_shared(web, shared).await.unwrap());
    }

    #[tokio::test]
    async fn deleting_a_copy_or_purging_the_owner_drops_outputs() {
        let s = store().await;
        let source = Uuid::new_v4();
        let copy = s.create_copy(O, source, None).await.unwrap().id;
        let master = AssetKey::master(source);
        s.put_export_output(output(copy, "").output(Uuid::new_v4(), None))
            .await
            .unwrap();
        s.put_export_output(output(master, "").output(Uuid::new_v4(), None))
            .await
            .unwrap();

        assert!(s.delete_copy(O, copy).await.unwrap());
        assert_eq!(s.get_export_output(output(copy, "")).await.unwrap(), None);
        assert!(
            s.get_export_output(output(master, ""))
                .await
                .unwrap()
                .is_some()
        );

        s.purge_owner(O).await.unwrap();
        assert_eq!(s.get_export_output(output(master, "")).await.unwrap(), None);
    }
}
