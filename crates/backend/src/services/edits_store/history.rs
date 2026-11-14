use chrono::Utc;
use raw_pipeline::edit_manifest::EditManifest;
use raw_pipeline::edits::Edits;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqliteConnection, SqliteExecutor};
use uuid::Uuid;

use super::*;

const HISTORY_LIMIT_PER_ASSET: i64 = 50;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditHistoryEntry {
    pub id: i64,
    pub manifest_hash: String,
    pub deleted: bool,
    pub edits: Option<Edits>,
    pub created_at: String,
    pub action: Option<String>,
    pub undone: bool,
}

#[derive(Debug)]
pub enum RestoreOutcome {
    Restored(EditRecord),
    Cleared,
    NotFound,
}

pub(super) struct NewHistoryEntry<'a> {
    pub manifest_hash: &'a str,
    pub edits_json: Option<&'a str>,
    pub deleted: bool,
    pub action: Option<&'a str>,
    pub created_at: &'a str,
}

fn history_from_row(row: &SqliteRow) -> Result<EditHistoryEntry, EditsStoreError> {
    let edits_json: Option<String> = row.try_get("edits_json")?;
    let edits = edits_json
        .as_deref()
        .and_then(|s| serde_json::from_str::<Edits>(s).ok());
    let deleted: i64 = row.try_get("deleted")?;
    let undone: i64 = row.try_get("undone")?;
    Ok(EditHistoryEntry {
        id: row.try_get("id")?,
        manifest_hash: row.try_get("manifest_hash")?,
        deleted: deleted != 0,
        edits,
        created_at: row.try_get("created_at")?,
        action: row.try_get("action")?,
        undone: undone != 0,
    })
}

fn unreadable_entry(entry_id: i64) -> EditsStoreError {
    EditsStoreError::Parse(<serde_json::Error as serde::de::Error>::custom(format!(
        "history entry {entry_id} has unreadable edits"
    )))
}

async fn fetch_history_entry<'e, E: SqliteExecutor<'e>>(
    exec: E,
    owner: Uuid,
    asset_id: AssetKey,
    entry_id: i64,
) -> Result<Option<EditHistoryEntry>, EditsStoreError> {
    let row = sqlx::query(
        "SELECT id, manifest_hash, edits_json, deleted, created_at, action, undone \
         FROM edits_history WHERE user_id = ?3 AND asset_id = ?1 AND id = ?2",
    )
    .bind(asset_id.to_string())
    .bind(entry_id)
    .bind(owner.to_string())
    .fetch_optional(exec)
    .await?;
    row.as_ref().map(history_from_row).transpose()
}

pub(super) async fn write_history(
    conn: &mut SqliteConnection,
    owner: Uuid,
    asset_id: AssetKey,
    entry: NewHistoryEntry<'_>,
) -> Result<(), EditsStoreError> {
    sqlx::query("DELETE FROM edits_history WHERE user_id = ?2 AND asset_id = ?1 AND undone = 1")
        .bind(asset_id.to_string())
        .bind(owner.to_string())
        .execute(&mut *conn)
        .await?;
    sqlx::query(
        "INSERT INTO edits_history (user_id, asset_id, manifest_hash, edits_json, deleted, created_at, action) \
         VALUES (?7, ?1, ?2, ?3, ?4, ?5, ?6)",
    )
    .bind(asset_id.to_string())
    .bind(entry.manifest_hash)
    .bind(entry.edits_json)
    .bind(if entry.deleted { 1 } else { 0 })
    .bind(entry.created_at)
    .bind(entry.action)
    .bind(owner.to_string())
    .execute(&mut *conn)
    .await?;
    sqlx::query(
        "DELETE FROM edits_history WHERE user_id = ?3 AND asset_id = ?1 AND id NOT IN (\
            SELECT id FROM edits_history WHERE user_id = ?3 AND asset_id = ?1 \
            ORDER BY id DESC LIMIT ?2\
         )",
    )
    .bind(asset_id.to_string())
    .bind(HISTORY_LIMIT_PER_ASSET)
    .bind(owner.to_string())
    .execute(&mut *conn)
    .await?;
    Ok(())
}

impl EditsStore {
    pub async fn list_history(
        &self,
        owner: Uuid,
        asset_id: AssetKey,
    ) -> Result<Vec<EditHistoryEntry>, EditsStoreError> {
        let rows = sqlx::query(
            "SELECT id, manifest_hash, edits_json, deleted, created_at, action, undone \
             FROM edits_history WHERE user_id = ?2 AND asset_id = ?1 ORDER BY id DESC",
        )
        .bind(asset_id.to_string())
        .bind(owner.to_string())
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(history_from_row).collect()
    }

    pub async fn restore_to_entry(
        &self,
        owner: Uuid,
        asset_id: AssetKey,
        entry_id: i64,
        immich_updated_at: Option<String>,
        immich_checksum: Option<String>,
    ) -> Result<RestoreOutcome, EditsStoreError> {
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        let Some(entry) = fetch_history_entry(&mut *tx, owner, asset_id, entry_id).await? else {
            return Ok(RestoreOutcome::NotFound);
        };
        let restored = match (entry.deleted, entry.edits) {
            (true, _) => None,
            (false, Some(edits)) => Some(edits.clamped()),
            (false, None) => return Err(unreadable_entry(entry_id)),
        };
        sqlx::query(
            "UPDATE edits_history SET undone = (id > ?2) WHERE user_id = ?3 AND asset_id = ?1",
        )
        .bind(asset_id.to_string())
        .bind(entry_id)
        .bind(owner.to_string())
        .execute(&mut *tx)
        .await?;
        let Some(edits) = restored else {
            sqlx::query("DELETE FROM edits WHERE user_id = ?2 AND asset_id = ?1")
                .bind(asset_id.to_string())
                .bind(owner.to_string())
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
            return Ok(RestoreOutcome::Cleared);
        };
        let edits_json = serde_json::to_string(&edits)?;
        let now = Utc::now().to_rfc3339();
        upsert_edits(
            &mut tx,
            owner,
            asset_id,
            &edits_json,
            immich_updated_at.as_deref(),
            immich_checksum.as_deref(),
            &now,
        )
        .await?;
        tx.commit().await?;
        Ok(RestoreOutcome::Restored(EditRecord {
            schema_version: SCHEMA_VERSION as u32,
            asset_id,
            immich_updated_at,
            immich_checksum,
            renderer_version: RENDERER_VERSION.to_string(),
            manifest: EditManifest::from_edits(&edits),
            updated_at: now,
            hash: edits.stable_hash(),
        }))
    }

    pub async fn get_history_entry(
        &self,
        owner: Uuid,
        asset_id: AssetKey,
        entry_id: i64,
    ) -> Result<Option<EditHistoryEntry>, EditsStoreError> {
        fetch_history_entry(&self.pool, owner, asset_id, entry_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{O, key, manifest_with_exposure, store, write};
    use super::*;

    async fn put_exposure(s: &EditsStore, id: AssetKey, ev: f64) -> i64 {
        s.put(O, id, write(manifest_with_exposure(ev)))
            .await
            .unwrap();
        s.list_history(O, id).await.unwrap()[0].id
    }

    async fn reset(s: &EditsStore, id: AssetKey) -> i64 {
        s.delete(O, id, None).await.unwrap();
        s.list_history(O, id).await.unwrap()[0].id
    }

    async fn timeline(s: &EditsStore, id: AssetKey) -> Vec<(Option<f64>, bool)> {
        s.list_history(O, id)
            .await
            .unwrap()
            .iter()
            .map(|e| (e.edits.as_ref().map(|x| x.basic.exposure_ev), e.undone))
            .collect()
    }

    async fn current_exposure(s: &EditsStore, id: AssetKey) -> Option<f64> {
        let record = s.get(O, id).await.unwrap();
        record.map(|r| r.manifest.to_edits().basic.exposure_ev)
    }

    async fn restore(s: &EditsStore, id: AssetKey, entry_id: i64) -> RestoreOutcome {
        s.restore_to_entry(O, id, entry_id, None, None)
            .await
            .unwrap()
    }

    fn restored_exposure(outcome: RestoreOutcome) -> f64 {
        match outcome {
            RestoreOutcome::Restored(record) => record.manifest.to_edits().basic.exposure_ev,
            other => panic!("expected restored edits: {other:?}"),
        }
    }

    fn manifest_with_depth(raster_id: &str) -> EditManifest {
        let mut edits = Edits::default();
        edits.lens_blur.depth_raster_id = raster_id.into();
        EditManifest::from_edits(&edits)
    }

    async fn pinned_rasters(s: &EditsStore, id: AssetKey) -> Vec<String> {
        sqlx::query_scalar("SELECT raster_id FROM raster_refs WHERE user_id = ?1 AND asset_id = ?2")
            .bind(O.to_string())
            .bind(id.to_string())
            .fetch_all(&s.pool)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn put_history_roundtrips_action() {
        let s = store().await;
        let id = key();
        s.put(
            O,
            id,
            EditWrite {
                manifest: manifest_with_exposure(0.5),
                action: Some("Exposure"),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        let hist = s.list_history(O, id).await.unwrap();
        if hist.len() != 1 || hist[0].action.as_deref() != Some("Exposure") {
            panic!("action not stored: {hist:?}");
        }
        s.put(O, id, EditWrite::default()).await.unwrap();
        let hist = s.list_history(O, id).await.unwrap();
        if hist.len() != 2 || hist[0].action.is_some() {
            panic!("missing null-action row: {hist:?}");
        }
    }

    #[tokio::test]
    async fn delete_history_roundtrips_action() {
        let s = store().await;
        let id = key();
        s.put(
            O,
            id,
            EditWrite {
                action: Some("Auto"),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        s.delete(O, id, Some("Brightness")).await.unwrap();
        let hist = s.list_history(O, id).await.unwrap();
        if !hist[0].deleted || hist[0].action.as_deref() != Some("Brightness") {
            panic!("tombstone action: {hist:?}");
        }
    }

    #[tokio::test]
    async fn history_reads_null_action() {
        let s = store().await;
        let id = key();
        sqlx::query(
            "INSERT INTO edits_history (asset_id, manifest_hash, edits_json, deleted, created_at) \
             VALUES (?1, ?2, ?3, 0, ?4)",
        )
        .bind(id.to_string())
        .bind("deadbeef")
        .bind("{}")
        .bind("2026-01-01T00:00:00Z")
        .execute(&s.pool)
        .await
        .unwrap();
        let hist = s.list_history(O, id).await.unwrap();
        if hist.len() != 1 || hist[0].action.is_some() || hist[0].manifest_hash != "deadbeef" {
            panic!("expected null action: {hist:?}");
        }
    }

    #[tokio::test]
    async fn history_rows_share_the_edit_timestamp() {
        let s = store().await;
        let id = key();
        let saved = s
            .put(O, id, write(manifest_with_exposure(0.5)))
            .await
            .unwrap();
        let hist = s.list_history(O, id).await.unwrap();
        if hist[0].created_at != saved.updated_at {
            panic!(
                "history {} vs edits {}",
                hist[0].created_at, saved.updated_at
            );
        }
    }

    #[tokio::test]
    async fn unchanged_put_after_restore_keeps_undone_entries() {
        let s = store().await;
        let id = key();
        let first = put_exposure(&s, id, 0.5).await;
        put_exposure(&s, id, 1.0).await;
        restore(&s, id, first).await;

        let metadata_only = EditWrite {
            manifest: manifest_with_exposure(0.5),
            immich_updated_at: Some("changed".into()),
            ..Default::default()
        };
        s.put(O, id, metadata_only).await.unwrap();
        let record = s.get(O, id).await.unwrap().unwrap();
        if record.immich_updated_at.as_deref() != Some("changed") {
            panic!("metadata not rewritten: {record:?}");
        }
        let entries = timeline(&s, id).await;
        if entries != [(Some(1.0), true), (Some(0.5), false)] {
            panic!("a metadata-only put discarded undone entries: {entries:?}");
        }
    }

    #[tokio::test]
    async fn reset_after_restore_discards_undone_entries() {
        let s = store().await;
        let id = key();
        let first = put_exposure(&s, id, 0.5).await;
        put_exposure(&s, id, 1.0).await;
        restore(&s, id, first).await;

        reset(&s, id).await;
        let entries = timeline(&s, id).await;
        if entries != [(None, false), (Some(0.5), false)] {
            panic!("undone entries survived a reset: {entries:?}");
        }
    }

    #[tokio::test]
    async fn restore_to_a_reset_keeps_newer_entries() {
        let s = store().await;
        let id = key();
        put_exposure(&s, id, 0.5).await;
        let cleared = reset(&s, id).await;
        let latest = put_exposure(&s, id, 1.0).await;

        let outcome = restore(&s, id, cleared).await;
        if !matches!(outcome, RestoreOutcome::Cleared) {
            panic!("expected cleared: {outcome:?}");
        }
        if current_exposure(&s, id).await.is_some() {
            panic!("edits row kept after restoring a reset");
        }
        let entries = timeline(&s, id).await;
        if entries != [(Some(1.0), true), (None, false), (Some(0.5), false)] {
            panic!("newer entry not kept as undone: {entries:?}");
        }

        if restored_exposure(restore(&s, id, latest).await) != 1.0 {
            panic!("forward restore returned the wrong state");
        }
        if current_exposure(&s, id).await != Some(1.0) {
            panic!("edits not brought back");
        }
        if timeline(&s, id).await.iter().any(|(_, undone)| *undone) {
            panic!("flags left after restoring the newest entry");
        }
    }

    #[tokio::test]
    async fn deleting_again_after_restoring_a_reset_keeps_undone_entries() {
        let s = store().await;
        let id = key();
        put_exposure(&s, id, 0.5).await;
        let cleared = reset(&s, id).await;
        put_exposure(&s, id, 1.0).await;
        restore(&s, id, cleared).await;

        if s.delete(O, id, None).await.unwrap() {
            panic!("a delete with no edits row reported a deletion");
        }
        let entries = timeline(&s, id).await;
        if entries != [(Some(1.0), true), (None, false), (Some(0.5), false)] {
            panic!("an empty delete touched the history: {entries:?}");
        }
    }

    #[tokio::test]
    async fn restoring_a_vanished_entry_is_not_found() {
        let s = store().await;
        let id = key();
        let first = put_exposure(&s, id, 0.5).await;
        let discarded = put_exposure(&s, id, 1.0).await;
        restore(&s, id, first).await;
        put_exposure(&s, id, 2.0).await;

        let outcome = restore(&s, id, discarded).await;
        if !matches!(outcome, RestoreOutcome::NotFound) {
            panic!("discarded entry restored: {outcome:?}");
        }
        let other_owner = s
            .restore_to_entry(Uuid::new_v4(), id, first, None, None)
            .await
            .unwrap();
        if !matches!(other_owner, RestoreOutcome::NotFound) {
            panic!("another owner restored the entry: {other_owner:?}");
        }
        if current_exposure(&s, id).await != Some(2.0) {
            panic!("a failed restore changed the edits");
        }
    }

    #[tokio::test]
    async fn restoring_an_unreadable_entry_fails_and_keeps_the_edits() {
        let s = store().await;
        let id = key();
        let first = put_exposure(&s, id, 0.5).await;
        put_exposure(&s, id, 1.0).await;
        sqlx::query("UPDATE edits_history SET edits_json = 'not json' WHERE id = ?1")
            .bind(first)
            .execute(&s.pool)
            .await
            .unwrap();

        let result = s.restore_to_entry(O, id, first, None, None).await;
        if !matches!(result, Err(EditsStoreError::Parse(_))) {
            panic!("expected a parse error: {result:?}");
        }
        if current_exposure(&s, id).await != Some(1.0) {
            panic!("an unreadable entry changed the edits");
        }
        if timeline(&s, id).await.iter().any(|(_, undone)| *undone) {
            panic!("a failed restore flagged entries");
        }
    }

    #[tokio::test]
    async fn restore_leaves_other_owners_history_alone() {
        let s = store().await;
        let id = key();
        let other = Uuid::new_v4();
        let first = put_exposure(&s, id, 0.5).await;
        put_exposure(&s, id, 1.0).await;
        for ev in [0.5, 1.0] {
            s.put(other, id, write(manifest_with_exposure(ev)))
                .await
                .unwrap();
        }
        let before = s.get(other, id).await.unwrap().unwrap();

        restore(&s, id, first).await;
        let theirs = s.list_history(other, id).await.unwrap();
        if theirs.len() != 2 || theirs.iter().any(|e| e.undone) {
            panic!("restore flagged another owner's history: {theirs:?}");
        }
        let after = s.get(other, id).await.unwrap().unwrap();
        if after.hash != before.hash || after.updated_at != before.updated_at {
            panic!("restore rewrote another owner's edits: {before:?} -> {after:?}");
        }
        if timeline(&s, id).await != [(Some(1.0), true), (Some(0.5), false)] {
            panic!("restore did not flag the owner's own history");
        }
    }

    #[tokio::test]
    async fn an_undone_entry_keeps_its_raster_pinned_until_discarded() {
        let s = store().await;
        let id = key();
        let first = put_exposure(&s, id, 0.5).await;
        s.put(O, id, write(manifest_with_depth("depth-r")))
            .await
            .unwrap();
        if pinned_rasters(&s, id).await != ["depth-r"] {
            panic!("raster not pinned by the current edits");
        }

        restore(&s, id, first).await;
        s.rebuild_raster_refs().await.unwrap();
        if pinned_rasters(&s, id).await != ["depth-r"] {
            panic!("raster of an undone entry was unpinned");
        }

        put_exposure(&s, id, 2.0).await;
        let pinned = pinned_rasters(&s, id).await;
        if !pinned.is_empty() {
            panic!("raster of a discarded entry still pinned: {pinned:?}");
        }
    }
}
