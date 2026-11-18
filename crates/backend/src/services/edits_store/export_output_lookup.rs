use uuid::Uuid;

use super::*;

#[derive(Debug, Clone, Copy)]
pub struct ExportOutputQuery<'a> {
    pub owner: Uuid,
    pub asset_ids: &'a [AssetKey],
    pub presets: &'a [String],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedExport {
    pub immich_asset_id: Uuid,
    pub exported_at: String,
}

impl ExportOutputQuery<'_> {
    fn asset_list(&self) -> Result<String, EditsStoreError> {
        let ids: Vec<String> = self.asset_ids.iter().map(AssetKey::to_string).collect();
        Ok(serde_json::to_string(&ids)?)
    }

    fn preset_list(&self) -> Result<String, EditsStoreError> {
        Ok(serde_json::to_string(self.presets)?)
    }
}

impl EditsStore {
    pub async fn count_exported_assets(
        &self,
        query: ExportOutputQuery<'_>,
    ) -> Result<u64, EditsStoreError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(DISTINCT asset_id) FROM export_outputs \
             WHERE user_id = ?1 \
             AND asset_id IN (SELECT value FROM json_each(?2)) \
             AND preset IN (SELECT value FROM json_each(?3))",
        )
        .bind(query.owner.to_string())
        .bind(query.asset_list()?)
        .bind(query.preset_list()?)
        .fetch_one(&self.pool)
        .await?;
        Ok(u64::try_from(count).unwrap_or_default())
    }

    pub async fn recorded_exports(
        &self,
        query: ExportOutputQuery<'_>,
    ) -> Result<Vec<RecordedExport>, EditsStoreError> {
        let rows: Vec<(Hyphenated, String)> = sqlx::query_as(
            "SELECT immich_asset_id, updated_at FROM export_outputs \
             WHERE user_id = ?1 \
             AND asset_id IN (SELECT value FROM json_each(?2)) \
             AND preset IN (SELECT value FROM json_each(?3)) \
             ORDER BY updated_at DESC",
        )
        .bind(query.owner.to_string())
        .bind(query.asset_list()?)
        .bind(query.preset_list()?)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|(id, exported_at)| RecordedExport {
                immich_asset_id: Uuid::from(id),
                exported_at,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{O, key, store};
    use super::*;

    const OTHER: Uuid = Uuid::from_u128(7);

    async fn track(s: &EditsStore, owner: Uuid, asset_id: AssetKey, preset: &str) -> Uuid {
        let id = Uuid::new_v4();
        let output = ExportOutputKey {
            owner,
            asset_id,
            preset,
        }
        .output(id, None);
        assert!(s.put_export_output(output).await.unwrap());
        id
    }

    fn presets(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| name.to_string()).collect()
    }

    fn query<'a>(asset_ids: &'a [AssetKey], presets: &'a [String]) -> ExportOutputQuery<'a> {
        ExportOutputQuery {
            owner: O,
            asset_ids,
            presets,
        }
    }

    #[tokio::test]
    async fn photos_count_once_whatever_presets_match() {
        let s = store().await;
        let [both, web, blank, none] = [(); 4].map(|_| key());
        track(&s, O, both, "web").await;
        track(&s, O, both, "print").await;
        track(&s, O, web, "web").await;
        track(&s, O, blank, "").await;
        let assets = [both, web, blank, none];

        for (names, want) in [
            (presets(&["web", "print"]), 2),
            (presets(&["print"]), 1),
            (presets(&[""]), 1),
            (presets(&["", "web"]), 3),
            (presets(&["other"]), 0),
        ] {
            let count = s.count_exported_assets(query(&assets, &names)).await;
            assert_eq!(count.unwrap(), want, "{names:?}");
        }
    }

    #[tokio::test]
    async fn another_users_outputs_are_not_counted_or_listed() {
        let s = store().await;
        let asset = key();
        track(&s, OTHER, asset, "").await;
        let blank = presets(&[""]);

        assert_eq!(
            s.count_exported_assets(query(&[asset], &blank))
                .await
                .unwrap(),
            0
        );
        assert!(
            s.recorded_exports(query(&[asset], &blank))
                .await
                .unwrap()
                .is_empty()
        );
    }

    async fn export_at(s: &EditsStore, id: Uuid, at: &str) {
        sqlx::query("UPDATE export_outputs SET updated_at = ?1 WHERE immich_asset_id = ?2")
            .bind(at)
            .bind(id.to_string())
            .execute(&s.pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn recorded_exports_list_matching_presets_newest_first() {
        let s = store().await;
        let asset = key();
        let newer = track(&s, O, asset, "print").await;
        let older = track(&s, O, asset, "web").await;
        export_at(&s, newer, "2026-11-02T08:00:00.5+00:00").await;
        export_at(&s, older, "2026-11-01T08:00:00+00:00").await;
        track(&s, O, asset, "").await;
        track(&s, O, key(), "web").await;

        let listed = s
            .recorded_exports(query(&[asset], &presets(&["web", "print"])))
            .await
            .unwrap();

        assert_eq!(
            listed,
            [
                RecordedExport {
                    immich_asset_id: newer,
                    exported_at: "2026-11-02T08:00:00.5+00:00".into(),
                },
                RecordedExport {
                    immich_asset_id: older,
                    exported_at: "2026-11-01T08:00:00+00:00".into(),
                },
            ]
        );
    }

    #[tokio::test]
    async fn thousands_of_ids_stay_one_query() {
        let s = store().await;
        let mut assets: Vec<AssetKey> = (0..10_000).map(|_| key()).collect();
        let tracked = assets[9_999];
        track(&s, O, tracked, "").await;
        assets.push(tracked);

        let count = s
            .count_exported_assets(query(&assets, &presets(&[""])))
            .await
            .unwrap();

        assert_eq!(count, 1);
    }
}
