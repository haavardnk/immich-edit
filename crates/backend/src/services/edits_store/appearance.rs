use std::collections::BTreeMap;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::*;

const DEFAULT_LIGHT_THEME: &str = "immich-edit-light";
const DEFAULT_DARK_THEME: &str = "immich-edit-dark";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(rename_all = "lowercase")]
pub enum AppearanceMode {
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
#[serde(deny_unknown_fields)]
pub struct Appearance {
    pub mode: AppearanceMode,
    pub light_theme: String,
    pub dark_theme: String,
    #[sqlx(rename = "accents_json", json)]
    pub accents: BTreeMap<String, String>,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            mode: AppearanceMode::Dark,
            light_theme: DEFAULT_LIGHT_THEME.to_string(),
            dark_theme: DEFAULT_DARK_THEME.to_string(),
            accents: BTreeMap::new(),
        }
    }
}

impl EditsStore {
    pub async fn get_appearance(&self, owner: Uuid) -> Result<Option<Appearance>, EditsStoreError> {
        Ok(sqlx::query_as::<_, Appearance>(
            "SELECT mode, light_theme, dark_theme, accents_json \
             FROM user_appearance WHERE user_id = ?1",
        )
        .bind(owner.to_string())
        .fetch_optional(&self.pool)
        .await?)
    }

    pub async fn put_appearance(
        &self,
        owner: Uuid,
        appearance: &Appearance,
    ) -> Result<Appearance, EditsStoreError> {
        sqlx::query(
            "INSERT INTO user_appearance \
             (user_id, mode, light_theme, dark_theme, accents_json, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
             ON CONFLICT(user_id) DO UPDATE SET \
               mode = excluded.mode, \
               light_theme = excluded.light_theme, \
               dark_theme = excluded.dark_theme, \
               accents_json = excluded.accents_json, \
               updated_at = excluded.updated_at",
        )
        .bind(owner.to_string())
        .bind(appearance.mode)
        .bind(&appearance.light_theme)
        .bind(&appearance.dark_theme)
        .bind(serde_json::to_string(&appearance.accents)?)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(appearance.clone())
    }
}
