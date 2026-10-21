use std::str::FromStr;

use sqlx::Row;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};

use super::*;

impl EditsStore {
    pub async fn connect(database_url: &str) -> Result<Self, EditsStoreError> {
        let opts = SqliteConnectOptions::from_str(database_url)?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(8)
            .connect_with(opts)
            .await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool })
    }

    pub async fn migrated_memory() -> Result<Self, EditsStoreError> {
        let opts = SqliteConnectOptions::from_str("sqlite::memory:")?
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(opts)
            .await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> SqlitePool {
        self.pool.clone()
    }

    pub async fn ready(&self) -> Result<(), EditsStoreError> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }

    pub async fn migration_version(&self) -> Result<Option<i64>, EditsStoreError> {
        let row = sqlx::query("SELECT MAX(version) AS v FROM _sqlx_migrations")
            .fetch_optional(&self.pool)
            .await?;
        let Some(row) = row else {
            return Ok(None);
        };
        Ok(row.try_get::<Option<i64>, _>("v")?)
    }
}

#[cfg(test)]
mod tests {
    use raw_pipeline::edits::{BasicEdits, WhiteBalance};

    use super::*;

    const O: Uuid = Uuid::nil();
    const LEGACY: &str = r#"{"basic":{"exposure_ev":0.5,"wb_temp":12.0,"wb_tint":7},
        "masks":[{"id":"l1","edits":{"wb_temp":3.0,"wb_tint":4.0}},{"id":"l2","edits":{}}]}"#;
    const AS_SHOT: &str = r#"{"basic":{"wb_temp":0.0,"wb_tint":0.0}}"#;
    const CURRENT: &str = r#"{"basic":{"white_balance":{"mode":"relative","temp":1.0,"tint":2.0}},"masks":[{"id":"l1","edits":{"wb_tint":5.0}}]}"#;

    async fn seed(s: &EditsStore, json: &str) -> AssetKey {
        let id = AssetKey::master(Uuid::new_v4());
        let edits = Edits {
            basic: BasicEdits {
                exposure_ev: 1.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let write = EditWrite {
            manifest: EditManifest::from_edits(&edits),
            ..Default::default()
        };
        s.put(O, id, write).await.unwrap();
        for sql in [
            "UPDATE edits SET edits_json = ?1 WHERE asset_id = ?2",
            "UPDATE edits_history SET edits_json = ?1 WHERE asset_id = ?2",
        ] {
            sqlx::query(sql)
                .bind(json)
                .bind(id.to_string())
                .execute(&s.pool)
                .await
                .unwrap();
        }
        id
    }

    async fn stored(s: &EditsStore, id: AssetKey) -> (Edits, Option<Edits>) {
        let current = s.get_edits_or_default(O, id).await.unwrap();
        let history = s.list_history(O, id).await.unwrap();
        (current, history.into_iter().next().and_then(|h| h.edits))
    }

    #[tokio::test]
    async fn white_balance_migration_rewrites_flat_fields() {
        let opts = SqliteConnectOptions::from_str("sqlite::memory:").unwrap();
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(opts)
            .await
            .unwrap();
        let migrator = sqlx::migrate!("./migrations");
        migrator.run_to(23, &pool).await.unwrap();
        let s = EditsStore { pool };
        let legacy = seed(&s, LEGACY).await;
        let as_shot = seed(&s, AS_SHOT).await;
        let current = seed(&s, CURRENT).await;
        migrator.run(&s.pool).await.unwrap();

        let (edits, history) = stored(&s, legacy).await;
        if history.as_ref() != Some(&edits) {
            panic!("history diverged: {history:?}");
        }
        let expected = WhiteBalance::Relative {
            temp: 12.0,
            tint: -7.0,
        };
        if edits.basic.white_balance != expected || edits.basic.exposure_ev != 0.5 {
            panic!("basic: {:?}", edits.basic);
        }
        let layers: Vec<(Option<f64>, Option<f64>)> = edits
            .masks
            .iter()
            .map(|l| (l.edits.wb_temp, l.edits.wb_tint))
            .collect();
        if layers != [(Some(3.0), Some(-4.0)), (None, None)] {
            panic!("masks: {layers:?}");
        }

        let (edits, history) = stored(&s, as_shot).await;
        let default = Edits::default().stable_hash();
        if edits.stable_hash() != default || history.map(|h| h.stable_hash()) != Some(default) {
            panic!("as-shot changed: {edits:?}");
        }

        let raw: String = sqlx::query_scalar("SELECT edits_json FROM edits WHERE asset_id = ?1")
            .bind(current.to_string())
            .fetch_one(&s.pool)
            .await
            .unwrap();
        if raw != CURRENT {
            panic!("current-format row rewritten: {raw}");
        }
    }

    async fn raw_json(s: &EditsStore, sql: &'static str, id: &str) -> serde_json::Value {
        let raw: String = sqlx::query_scalar(sql)
            .bind(id)
            .fetch_one(&s.pool)
            .await
            .unwrap();
        serde_json::from_str(&raw).unwrap()
    }

    #[tokio::test]
    async fn mask_color_migration_strips_layer_colors() {
        const COLORED: &str = r##"{"masks":[{"id":"l1","color":"#ff9500","amount":0.5},{"id":"l2","color":"#ff3b30","edits":{"exposure_ev":1.0}}]}"##;
        const PRESET: &str = r##"{"schema_version":5,"ops":{"masks":{"layers":[{"id":"l1","color":"#ff9500","amount":0.5},"junk"]}}}"##;
        let opts = SqliteConnectOptions::from_str("sqlite::memory:").unwrap();
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(opts)
            .await
            .unwrap();
        let migrator = sqlx::migrate!("./migrations");
        migrator.run_to(25, &pool).await.unwrap();
        let s = EditsStore { pool };
        let colored = seed(&s, COLORED).await.to_string();
        let current = seed(&s, CURRENT).await.to_string();
        let preset = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO presets (id, name, manifest_json, schema_version, created_at, \
             updated_at, user_id) VALUES (?1, 'p', ?2, 5, 't', 't', ?3)",
        )
        .bind(&preset)
        .bind(PRESET)
        .bind(O.to_string())
        .execute(&s.pool)
        .await
        .unwrap();
        migrator.run(&s.pool).await.unwrap();

        let layers = serde_json::json!([
            {"id": "l1", "amount": 0.5},
            {"id": "l2", "edits": {"exposure_ev": 1.0}}
        ]);
        let edits_sql = "SELECT edits_json FROM edits WHERE asset_id = ?1";
        let history_sql = "SELECT edits_json FROM edits_history WHERE asset_id = ?1";
        for sql in [edits_sql, history_sql] {
            let json = raw_json(&s, sql, &colored).await;
            if json["masks"] != layers {
                panic!("colors kept: {json}");
            }
        }
        let json = raw_json(
            &s,
            "SELECT manifest_json FROM presets WHERE id = ?1",
            &preset,
        )
        .await;
        if json["ops"]["masks"]["layers"]
            != serde_json::json!([{"id": "l1", "amount": 0.5}, "junk"])
        {
            panic!("preset colors kept: {json}");
        }
        let raw: String = sqlx::query_scalar(edits_sql)
            .bind(&current)
            .fetch_one(&s.pool)
            .await
            .unwrap();
        if raw != CURRENT {
            panic!("colorless row rewritten: {raw}");
        }
    }
}
