use std::collections::BTreeMap;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqliteExecutor;
use sqlx::types::Json;
use uuid::Uuid;

use super::*;

mod patch;

pub use patch::PreferencesPatch;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SectionEntry {
    pub id: String,
    pub hidden: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CropRatio {
    pub num: i64,
    pub den: i64,
}

impl CropRatio {
    pub fn normalized(self) -> Self {
        if self.num < self.den {
            Self {
                num: self.den,
                den: self.num,
            }
        } else {
            self
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TileOverlay {
    pub icons: Vec<String>,
    pub details: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, sqlx::FromRow)]
pub struct Preferences {
    #[sqlx(rename = "pinned_settings_json", json(nullable))]
    pub pinned_settings: Option<Vec<String>>,
    pub mask_overlay_color: Option<String>,
    #[sqlx(rename = "shortcuts_json", json)]
    pub shortcuts: BTreeMap<String, Vec<String>>,
    #[sqlx(rename = "desktop_sections_json", json(nullable))]
    pub desktop_sections: Option<Vec<SectionEntry>>,
    #[sqlx(rename = "touch_subviews_json", json(nullable))]
    pub touch_subviews: Option<Vec<SectionEntry>>,
    #[sqlx(rename = "crop_ratios_json", json(nullable))]
    pub crop_ratios: Option<Vec<CropRatio>>,
    #[sqlx(rename = "tile_overlay_json", json(nullable))]
    pub tile_overlay: Option<TileOverlay>,
}

async fn fetch_preferences<'e, E: SqliteExecutor<'e>>(
    exec: E,
    owner: Uuid,
) -> Result<Option<Preferences>, EditsStoreError> {
    Ok(sqlx::query_as::<_, Preferences>(
        "SELECT mask_overlay_color, pinned_settings_json, shortcuts_json, \
         desktop_sections_json, touch_subviews_json, crop_ratios_json, tile_overlay_json \
         FROM user_preferences WHERE user_id = ?1",
    )
    .bind(owner.to_string())
    .fetch_optional(exec)
    .await?)
}

async fn upsert_preferences<'e, E: SqliteExecutor<'e>>(
    exec: E,
    owner: Uuid,
    preferences: &Preferences,
) -> Result<(), EditsStoreError> {
    sqlx::query(
        "INSERT INTO user_preferences \
         (user_id, mask_overlay_color, pinned_settings_json, shortcuts_json, \
          desktop_sections_json, touch_subviews_json, crop_ratios_json, tile_overlay_json, \
          updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9) \
         ON CONFLICT(user_id) DO UPDATE SET \
           mask_overlay_color = excluded.mask_overlay_color, \
           pinned_settings_json = excluded.pinned_settings_json, \
           shortcuts_json = excluded.shortcuts_json, \
           desktop_sections_json = excluded.desktop_sections_json, \
           touch_subviews_json = excluded.touch_subviews_json, \
           crop_ratios_json = excluded.crop_ratios_json, \
           tile_overlay_json = excluded.tile_overlay_json, \
           updated_at = excluded.updated_at",
    )
    .bind(owner.to_string())
    .bind(&preferences.mask_overlay_color)
    .bind(preferences.pinned_settings.as_ref().map(Json))
    .bind(Json(&preferences.shortcuts))
    .bind(preferences.desktop_sections.as_ref().map(Json))
    .bind(preferences.touch_subviews.as_ref().map(Json))
    .bind(preferences.crop_ratios.as_ref().map(Json))
    .bind(preferences.tile_overlay.as_ref().map(Json))
    .bind(Utc::now().to_rfc3339())
    .execute(exec)
    .await?;
    Ok(())
}

impl EditsStore {
    pub async fn get_preferences(
        &self,
        owner: Uuid,
    ) -> Result<Option<Preferences>, EditsStoreError> {
        fetch_preferences(&self.pool, owner).await
    }

    pub async fn update_preferences(
        &self,
        owner: Uuid,
        patch: PreferencesPatch,
    ) -> Result<Preferences, EditsStoreError> {
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        let current = fetch_preferences(&mut *tx, owner)
            .await?
            .unwrap_or_default();
        let next = patch.apply(current);
        upsert_preferences(&mut *tx, owner, &next).await?;
        tx.commit().await?;
        Ok(next)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::tests::{O, store};
    use super::*;

    fn section(id: &str, hidden: bool) -> SectionEntry {
        SectionEntry {
            id: id.into(),
            hidden,
        }
    }

    async fn patch(s: &EditsStore, owner: Uuid, body: serde_json::Value) -> Preferences {
        s.update_preferences(owner, serde_json::from_value(body).unwrap())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn nullable_columns_store_sql_null() {
        let s = store().await;
        patch(
            &s,
            O,
            json!({
                "pinned_settings": ["exposure"],
                "crop_ratios": [{ "num": 3, "den": 2 }],
                "tile_overlay": { "icons": ["favorite"], "details": [] }
            }),
        )
        .await;
        patch(
            &s,
            O,
            json!({ "pinned_settings": null, "crop_ratios": null, "tile_overlay": null }),
        )
        .await;
        let stored: (Option<String>, Option<String>, Option<String>) = sqlx::query_as(
            "SELECT pinned_settings_json, crop_ratios_json, tile_overlay_json \
             FROM user_preferences WHERE user_id = ?1",
        )
        .bind(O.to_string())
        .fetch_one(&s.pool)
        .await
        .unwrap();
        assert_eq!(stored, (None, None, None));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_patches_to_different_fields_all_land() {
        let dir = tempfile::tempdir().unwrap();
        let url = format!("sqlite://{}/edits.db", dir.path().display());
        let s = EditsStore::connect(&url).await.unwrap();
        let bodies = [
            json!({ "pinned_settings": ["exposure"] }),
            json!({ "mask_overlay_color": "#123456" }),
            json!({ "shortcuts": { "help": ["F1"] } }),
            json!({ "desktop_sections": [{ "id": "basic", "hidden": true }] }),
            json!({ "touch_subviews": [{ "id": "light", "hidden": false }] }),
            json!({ "crop_ratios": [{ "num": 5, "den": 4 }] }),
            json!({ "tile_overlay": { "icons": ["location"], "details": ["camera"] } }),
            json!({ "pinned_settings": ["exposure"], "crop_ratios": [{ "num": 5, "den": 4 }] }),
            json!({ "shortcuts": { "help": ["F1"] }, "mask_overlay_color": "#123456" }),
        ];

        let writers = bodies.map(|body| {
            let store = s.clone();
            tokio::spawn(async move {
                store
                    .update_preferences(O, serde_json::from_value(body).unwrap())
                    .await
                    .unwrap()
            })
        });
        for outcome in futures_util::future::join_all(writers).await {
            outcome.unwrap();
        }

        assert_eq!(
            s.get_preferences(O).await.unwrap(),
            Some(Preferences {
                pinned_settings: Some(vec!["exposure".into()]),
                mask_overlay_color: Some("#123456".into()),
                shortcuts: BTreeMap::from([("help".into(), vec!["F1".into()])]),
                desktop_sections: Some(vec![section("basic", true)]),
                touch_subviews: Some(vec![section("light", false)]),
                crop_ratios: Some(vec![CropRatio { num: 5, den: 4 }]),
                tile_overlay: Some(TileOverlay {
                    icons: vec!["location".into()],
                    details: vec!["camera".into()],
                }),
            })
        );
    }
}
