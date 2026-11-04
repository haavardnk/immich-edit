use std::collections::BTreeMap;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqliteExecutor;
use sqlx::types::Json;
use uuid::Uuid;

use super::*;

mod patch;

pub use patch::PreferencesPatch;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(rename_all = "lowercase")]
pub enum ClickTarget {
    #[default]
    Editor,
    Loupe,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SectionEntry {
    pub id: String,
    pub hidden: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, sqlx::FromRow)]
pub struct Preferences {
    pub click_target: ClickTarget,
    #[sqlx(rename = "pinned_settings_json", json(nullable))]
    pub pinned_settings: Option<Vec<String>>,
    pub mask_overlay_color: Option<String>,
    #[sqlx(rename = "shortcuts_json", json)]
    pub shortcuts: BTreeMap<String, Vec<String>>,
    #[sqlx(rename = "desktop_sections_json", json(nullable))]
    pub desktop_sections: Option<Vec<SectionEntry>>,
    #[sqlx(rename = "touch_subviews_json", json(nullable))]
    pub touch_subviews: Option<Vec<SectionEntry>>,
}

async fn fetch_preferences<'e, E: SqliteExecutor<'e>>(
    exec: E,
    owner: Uuid,
) -> Result<Option<Preferences>, EditsStoreError> {
    Ok(sqlx::query_as::<_, Preferences>(
        "SELECT click_target, mask_overlay_color, pinned_settings_json, shortcuts_json, \
         desktop_sections_json, touch_subviews_json \
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
         (user_id, click_target, mask_overlay_color, pinned_settings_json, shortcuts_json, \
          desktop_sections_json, touch_subviews_json, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8) \
         ON CONFLICT(user_id) DO UPDATE SET \
           click_target = excluded.click_target, \
           mask_overlay_color = excluded.mask_overlay_color, \
           pinned_settings_json = excluded.pinned_settings_json, \
           shortcuts_json = excluded.shortcuts_json, \
           desktop_sections_json = excluded.desktop_sections_json, \
           touch_subviews_json = excluded.touch_subviews_json, \
           updated_at = excluded.updated_at",
    )
    .bind(owner.to_string())
    .bind(preferences.click_target)
    .bind(&preferences.mask_overlay_color)
    .bind(preferences.pinned_settings.as_ref().map(Json))
    .bind(Json(&preferences.shortcuts))
    .bind(preferences.desktop_sections.as_ref().map(Json))
    .bind(preferences.touch_subviews.as_ref().map(Json))
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

    fn everything() -> Preferences {
        Preferences {
            click_target: ClickTarget::Loupe,
            pinned_settings: Some(vec!["exposure".into(), "white_balance".into()]),
            mask_overlay_color: Some("#ff8800".into()),
            shortcuts: BTreeMap::from([
                ("help".into(), vec!["?".into(), "Shift+/".into()]),
                ("editor.copy".into(), vec![]),
            ]),
            desktop_sections: Some(vec![section("basic", false), section("detail", true)]),
            touch_subviews: Some(vec![section("light", true)]),
        }
    }

    fn everything_patch() -> serde_json::Value {
        json!({
            "click_target": "loupe",
            "pinned_settings": ["exposure", "white_balance"],
            "mask_overlay_color": "#ff8800",
            "shortcuts": { "help": ["?", "Shift+/"], "editor.copy": [] },
            "desktop_sections": [
                { "id": "basic", "hidden": false },
                { "id": "detail", "hidden": true }
            ],
            "touch_subviews": [{ "id": "light", "hidden": true }]
        })
    }

    async fn patch(s: &EditsStore, owner: Uuid, body: serde_json::Value) -> Preferences {
        s.update_preferences(owner, serde_json::from_value(body).unwrap())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn no_row_reads_as_none_and_merges_onto_defaults() {
        let s = store().await;
        assert_eq!(s.get_preferences(O).await.unwrap(), None);
        let saved = patch(&s, O, json!({})).await;
        assert_eq!(saved, Preferences::default());
        assert_eq!(s.get_preferences(O).await.unwrap(), Some(saved));
    }

    #[tokio::test]
    async fn every_field_round_trips() {
        let s = store().await;
        assert_eq!(patch(&s, O, everything_patch()).await, everything());
        assert_eq!(s.get_preferences(O).await.unwrap(), Some(everything()));
    }

    #[tokio::test]
    async fn nullable_columns_store_sql_null() {
        let s = store().await;
        patch(&s, O, everything_patch()).await;
        patch(&s, O, json!({ "pinned_settings": null })).await;
        let stored: Option<String> = sqlx::query_scalar(
            "SELECT pinned_settings_json FROM user_preferences WHERE user_id = ?1",
        )
        .bind(O.to_string())
        .fetch_one(&s.pool)
        .await
        .unwrap();
        assert_eq!(stored, None);
    }

    #[tokio::test]
    async fn owners_do_not_share_preferences() {
        let s = store().await;
        let other = Uuid::new_v4();
        patch(&s, O, json!({ "click_target": "loupe" })).await;
        assert_eq!(s.get_preferences(other).await.unwrap(), None);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_patches_to_different_fields_all_land() {
        let dir = tempfile::tempdir().unwrap();
        let url = format!("sqlite://{}/edits.db", dir.path().display());
        let s = EditsStore::connect(&url).await.unwrap();
        let bodies = [
            json!({ "click_target": "loupe" }),
            json!({ "pinned_settings": ["exposure"] }),
            json!({ "mask_overlay_color": "#123456" }),
            json!({ "shortcuts": { "help": ["F1"] } }),
            json!({ "desktop_sections": [{ "id": "basic", "hidden": true }] }),
            json!({ "touch_subviews": [{ "id": "light", "hidden": false }] }),
            json!({ "pinned_settings": ["exposure"], "click_target": "loupe" }),
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
                click_target: ClickTarget::Loupe,
                pinned_settings: Some(vec!["exposure".into()]),
                mask_overlay_color: Some("#123456".into()),
                shortcuts: BTreeMap::from([("help".into(), vec!["F1".into()])]),
                desktop_sections: Some(vec![section("basic", true)]),
                touch_subviews: Some(vec![section("light", false)]),
            })
        );
    }
}
