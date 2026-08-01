use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use chrono::Utc;
use raw_pipeline::dcp::{DCP_MAX_SOURCE_BYTES, DcpProfile};
use raw_pipeline::parse_dcp;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{Row, SqlitePool};
use tokio::fs;
use uuid::Uuid;

use super::blob_store;

#[derive(Debug, thiserror::Error)]
pub enum DcpStoreError {
    #[error("db: {0}")]
    Db(#[from] sqlx::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid: {0}")]
    Invalid(String),
    #[error("duplicate")]
    Duplicate(Box<DcpMeta>),
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DcpMeta {
    pub id: String,
    pub name: String,
    pub camera_model: Option<String>,
    pub copyright: Option<String>,
    pub bundled: bool,
    pub size: u64,
    pub created_at: String,
}

#[derive(Clone)]
pub struct DcpStore {
    pool: SqlitePool,
    dir: PathBuf,
    cache: Arc<Mutex<HashMap<String, Arc<DcpProfile>>>>,
}

struct DcpRecord {
    meta: DcpMeta,
    content_hash: String,
}

#[derive(Deserialize)]
struct BundledManifest {
    profiles: Vec<BundledEntry>,
}

#[derive(Deserialize)]
struct BundledEntry {
    file: String,
    camera_model: Option<String>,
}

async fn bundled_camera_models(dir: &Path) -> HashMap<String, String> {
    let Ok(bytes) = fs::read(dir.join("manifest.json")).await else {
        return HashMap::new();
    };
    match serde_json::from_slice::<BundledManifest>(&bytes) {
        Ok(manifest) => manifest
            .profiles
            .into_iter()
            .filter_map(|entry| entry.camera_model.map(|model| (entry.file, model)))
            .collect(),
        Err(e) => {
            tracing::warn!(error = %e, "bundled dcp manifest unreadable");
            HashMap::new()
        }
    }
}

impl DcpStore {
    pub fn new(pool: SqlitePool, cache_dir: &Path) -> Result<Self, DcpStoreError> {
        let dir = cache_dir.join("dcp");
        std::fs::create_dir_all(&dir)?;
        Ok(Self {
            pool,
            dir,
            cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    fn blob_path(&self, content_hash: &str) -> PathBuf {
        self.dir.join(format!("{content_hash}.dcp"))
    }

    fn row_to_meta(row: &sqlx::sqlite::SqliteRow) -> DcpMeta {
        DcpMeta {
            id: row.get("id"),
            name: row.get("name"),
            camera_model: row.get("camera_model"),
            copyright: row.get("copyright"),
            bundled: row.get::<i64, _>("bundled") != 0,
            size: row.get::<i64, _>("size") as u64,
            created_at: row.get("created_at"),
        }
    }

    pub async fn import(
        &self,
        name: Option<&str>,
        camera: Option<&str>,
        bundled: bool,
        bytes: &[u8],
    ) -> Result<DcpMeta, DcpStoreError> {
        if bytes.len() > DCP_MAX_SOURCE_BYTES {
            return Err(DcpStoreError::Invalid("dcp source too large".into()));
        }
        let content_hash = blob_store::content_hash(bytes);
        if let Some(existing) = self.find_active_hash(&content_hash).await? {
            return Err(DcpStoreError::Duplicate(Box::new(existing)));
        }
        let profile = parse_dcp(bytes).map_err(|e| DcpStoreError::Invalid(e.to_string()))?;
        let name = name
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .or_else(|| profile.name.clone())
            .unwrap_or_else(|| "DCP profile".to_string());
        let camera_model = camera
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .or_else(|| profile.unique_camera_model.clone());
        let copyright = profile.copyright.clone();
        blob_store::write_blob_atomic(&self.blob_path(&content_hash), bytes).await?;
        self.cache
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .entry(content_hash.clone())
            .or_insert_with(|| Arc::new(profile));

        let id = Uuid::new_v4().to_string();
        let created_at = Utc::now().to_rfc3339();
        let size = bytes.len() as i64;
        if let Err(e) = sqlx::query(
            "INSERT INTO dcp_profiles (id, name, camera_model, copyright, content_hash, size, bundled, deleted, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, 0, ?)",
        )
        .bind(&id)
        .bind(&name)
        .bind(&camera_model)
        .bind(&copyright)
        .bind(&content_hash)
        .bind(size)
        .bind(bundled as i64)
        .bind(&created_at)
        .execute(&self.pool)
        .await
        {
            if blob_store::is_unique_violation(&e)
                && let Some(existing) = self.find_active_hash(&content_hash).await?
            {
                return Err(DcpStoreError::Duplicate(Box::new(existing)));
            }
            return Err(e.into());
        }

        Ok(DcpMeta {
            id,
            name,
            camera_model,
            copyright,
            bundled,
            size: bytes.len() as u64,
            created_at,
        })
    }

    pub async fn import_bundled(&self, dir: &Path) -> Result<usize, DcpStoreError> {
        if !fs::try_exists(dir).await? {
            return Ok(0);
        }
        let manifest_models = bundled_camera_models(dir).await;
        let mut count = 0;
        let mut rd = fs::read_dir(dir).await?;
        while let Some(entry) = rd.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("dcp") {
                continue;
            }
            let bytes = fs::read(&path).await?;
            let camera = path
                .file_name()
                .and_then(|name| name.to_str())
                .and_then(|name| manifest_models.get(name))
                .map(String::as_str);
            match self.import(None, camera, true, &bytes).await {
                Ok(_) => count += 1,
                Err(DcpStoreError::Duplicate(_)) => {}
                Err(DcpStoreError::Invalid(reason)) => {
                    tracing::warn!(
                        path = %path.display(),
                        reason,
                        "skipped invalid bundled dcp profile"
                    );
                }
                Err(e) => return Err(e),
            }
        }
        Ok(count)
    }

    pub async fn list(&self) -> Result<Vec<DcpMeta>, DcpStoreError> {
        let rows = sqlx::query(
            "SELECT id, name, camera_model, copyright, bundled, size, created_at FROM dcp_profiles WHERE deleted = 0 ORDER BY bundled ASC, name ASC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.iter().map(Self::row_to_meta).collect())
    }

    pub async fn revision(&self) -> Result<String, DcpStoreError> {
        let rows = sqlx::query(
            "SELECT id, content_hash, camera_model, bundled FROM dcp_profiles WHERE deleted = 0 ORDER BY id ASC",
        )
        .fetch_all(&self.pool)
        .await?;
        let mut digest = Sha256::new();
        for row in rows {
            digest.update(row.get::<String, _>("id"));
            digest.update(row.get::<String, _>("content_hash"));
            digest.update(
                row.get::<Option<String>, _>("camera_model")
                    .unwrap_or_default(),
            );
            digest.update(row.get::<i64, _>("bundled").to_le_bytes());
        }
        Ok(hex::encode(digest.finalize()))
    }

    async fn find_active_hash(&self, content_hash: &str) -> Result<Option<DcpMeta>, DcpStoreError> {
        let row = sqlx::query(
            "SELECT id, name, camera_model, copyright, bundled, size, created_at FROM dcp_profiles WHERE content_hash = ? AND deleted = 0",
        )
        .bind(content_hash)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.as_ref().map(Self::row_to_meta))
    }

    pub async fn soft_delete(&self, id: &str) -> Result<(), DcpStoreError> {
        let affected =
            sqlx::query("UPDATE dcp_profiles SET deleted = 1 WHERE id = ? AND deleted = 0")
                .bind(id)
                .execute(&self.pool)
                .await?
                .rows_affected();
        if affected == 0 {
            return Err(DcpStoreError::NotFound);
        }
        Ok(())
    }

    pub async fn load(&self, id: &str) -> Result<Arc<DcpProfile>, DcpStoreError> {
        let row = sqlx::query("SELECT content_hash FROM dcp_profiles WHERE id = ? AND deleted = 0")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(DcpStoreError::NotFound)?;
        let content_hash: String = row.get("content_hash");
        self.load_hash(&content_hash).await
    }

    pub async fn match_camera(
        &self,
        model: &str,
    ) -> Result<Option<Arc<DcpProfile>>, DcpStoreError> {
        match self.match_camera_record(model).await? {
            Some(record) => Ok(Some(self.load_hash(&record.content_hash).await?)),
            None => Ok(None),
        }
    }

    pub async fn match_camera_meta(&self, model: &str) -> Result<Option<DcpMeta>, DcpStoreError> {
        Ok(self
            .match_camera_record(model)
            .await?
            .map(|record| record.meta))
    }

    async fn match_camera_record(&self, model: &str) -> Result<Option<DcpRecord>, DcpStoreError> {
        let needle = normalize_model(model);
        if needle.is_empty() {
            return Ok(None);
        }
        let rows = sqlx::query(
            "SELECT id, name, camera_model, copyright, content_hash, bundled, size, created_at FROM dcp_profiles WHERE camera_model IS NOT NULL AND deleted = 0 ORDER BY bundled ASC, created_at DESC, name ASC, id ASC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.iter().find_map(|row| {
            let camera: String = row.get("camera_model");
            models_match(&camera, &needle).then(|| DcpRecord {
                meta: Self::row_to_meta(row),
                content_hash: row.get("content_hash"),
            })
        }))
    }

    async fn load_hash(&self, content_hash: &str) -> Result<Arc<DcpProfile>, DcpStoreError> {
        if let Some(p) = self
            .cache
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(content_hash)
        {
            return Ok(p.clone());
        }
        let bytes = fs::read(self.blob_path(content_hash)).await?;
        let profile = parse_dcp(&bytes).map_err(|e| DcpStoreError::Invalid(e.to_string()))?;
        let profile = Arc::new(profile);
        self.cache
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(content_hash.to_string(), profile.clone());
        Ok(profile)
    }
}

fn normalize_model(s: &str) -> String {
    s.trim()
        .to_ascii_uppercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

const KNOWN_MAKES: &[&str] = &[
    "NIKONCORPORATION",
    "OMDIGITALSOLUTIONS",
    "RICOHIMAGINGCOMPANYLTD",
    "EASTMANKODAKCOMPANY",
    "LEICACAMERAAG",
    "PANASONIC",
    "HASSELBLAD",
    "FUJIFILM",
    "OLYMPUS",
    "SAMSUNG",
    "PENTAX",
    "CANON",
    "NIKON",
    "LEICA",
    "KODAK",
    "SIGMA",
    "RICOH",
    "SONY",
];

const MODEL_ALIASES: &[(&str, &str)] = &[("Z62", "Z6II"), ("Z72", "Z7II")];

fn strip_make(normalized: &str) -> &str {
    KNOWN_MAKES
        .iter()
        .find_map(|make| {
            normalized
                .strip_prefix(make)
                .filter(|rest| !rest.is_empty())
        })
        .unwrap_or(normalized)
}

fn aliased(a: &str, b: &str) -> bool {
    MODEL_ALIASES
        .iter()
        .any(|&(x, y)| (a == x && b == y) || (a == y && b == x))
}

fn models_match(camera: &str, exif_normalized: &str) -> bool {
    let camera = normalize_model(camera);
    if camera == exif_normalized {
        return true;
    }
    let camera_core = strip_make(&camera);
    let exif_core = strip_make(exif_normalized);
    camera_core == exif_core || aliased(camera_core, exif_core)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::edits_store::EditsStore;
    use std::collections::BTreeSet;

    fn build_dcp(model: Option<&str>) -> Vec<u8> {
        build_named_dcp(model, None)
    }

    fn build_named_dcp(model: Option<&str>, name: Option<&str>) -> Vec<u8> {
        let mut entries: Vec<(u16, u16, u32, Vec<u8>)> = Vec::new();
        let cm: [f32; 9] = [0.6, -0.1, -0.05, -0.3, 1.2, 0.1, 0.02, -0.2, 0.9];
        let mut srational = Vec::new();
        for v in cm {
            srational.extend_from_slice(&((v * 10_000.0).round() as i32).to_le_bytes());
            srational.extend_from_slice(&10_000i32.to_le_bytes());
        }
        entries.push((50721, 10, 9, srational));
        if let Some(m) = model {
            let mut b = m.as_bytes().to_vec();
            b.push(0);
            let count = b.len() as u32;
            entries.push((50708, 2, count, b));
        }
        if let Some(n) = name {
            let mut b = n.as_bytes().to_vec();
            b.push(0);
            let count = b.len() as u32;
            entries.push((50936, 2, count, b));
        }
        entries.sort_by_key(|e| e.0);
        let n = entries.len();
        let ifd_off = 8usize;
        let data_off = ifd_off + 2 + n * 12 + 4;
        let mut out = Vec::new();
        out.extend_from_slice(b"II");
        out.extend_from_slice(&42u16.to_le_bytes());
        out.extend_from_slice(&(ifd_off as u32).to_le_bytes());
        out.extend_from_slice(&(n as u16).to_le_bytes());
        let mut blob = Vec::new();
        for (tag, typ, count, bytes) in &entries {
            out.extend_from_slice(&tag.to_le_bytes());
            out.extend_from_slice(&typ.to_le_bytes());
            out.extend_from_slice(&count.to_le_bytes());
            if bytes.len() <= 4 {
                let mut field = bytes.clone();
                field.resize(4, 0);
                out.extend_from_slice(&field);
            } else {
                let off = data_off + blob.len();
                out.extend_from_slice(&(off as u32).to_le_bytes());
                blob.extend_from_slice(bytes);
            }
        }
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&blob);
        out
    }

    async fn store() -> (DcpStore, tempfile::TempDir) {
        let edits = EditsStore::migrated_memory().await.unwrap();
        let dir = tempfile::tempdir().unwrap();
        let store = DcpStore::new(edits.pool(), dir.path()).unwrap();
        (store, dir)
    }

    #[tokio::test]
    async fn import_list_load_delete_roundtrip() {
        let (store, _dir) = store().await;
        let bytes = build_dcp(Some("SONY ILCE-7M4"));
        let meta = store
            .import(Some("My Profile"), None, false, &bytes)
            .await
            .unwrap();
        assert_eq!(meta.name, "My Profile");
        assert_eq!(meta.camera_model.as_deref(), Some("SONY ILCE-7M4"));
        assert!(!meta.bundled);

        let listed = store.list().await.unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, meta.id);

        store.load(&meta.id).await.unwrap();

        store.soft_delete(&meta.id).await.unwrap();
        assert!(store.list().await.unwrap().is_empty());
        assert!(matches!(
            store.load(&meta.id).await,
            Err(DcpStoreError::NotFound)
        ));
    }

    #[tokio::test]
    async fn import_rejects_duplicate_hash() {
        let (store, _dir) = store().await;
        let bytes = build_dcp(Some("NIKON Z6"));
        store.import(None, None, false, &bytes).await.unwrap();
        let err = store.import(None, None, false, &bytes).await.unwrap_err();
        assert!(matches!(err, DcpStoreError::Duplicate(_)));
    }

    #[tokio::test]
    async fn match_camera_normalizes_model() {
        let (store, _dir) = store().await;
        let bytes = build_dcp(Some("SONY ILCE-7M4"));
        store.import(None, None, false, &bytes).await.unwrap();
        assert!(store.match_camera("ILCE-7M4").await.unwrap().is_some());
        assert!(store.match_camera("Canon EOS R5").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn match_camera_rejects_partial_suffix() {
        let (store, _dir) = store().await;
        let bytes = build_dcp(Some("7M4"));
        store.import(None, None, false, &bytes).await.unwrap();
        assert!(store.match_camera("ILCE-7M4").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn match_camera_uses_model_alias() {
        let (store, _dir) = store().await;
        let bytes = build_dcp(Some("NIKON Z 6_2"));
        store.import(None, None, false, &bytes).await.unwrap();
        assert!(store.match_camera("NIKON Z 6_2").await.unwrap().is_some());
        assert!(store.match_camera("Z6II").await.unwrap().is_some());
    }

    #[tokio::test]
    async fn match_camera_prefers_imported_profile() {
        let (store, _dir) = store().await;
        let bundled = build_named_dcp(Some("SONY ILCE-7M4"), Some("Bundled"));
        let imported = build_named_dcp(Some("SONY ILCE-7M4"), Some("Imported"));
        store.import(None, None, true, &bundled).await.unwrap();
        let imported_meta = store.import(None, None, false, &imported).await.unwrap();
        let matched = store
            .match_camera_meta("ILCE-7M4")
            .await
            .unwrap()
            .expect("camera match");
        assert_eq!(matched.id, imported_meta.id);
        assert!(!matched.bundled);
    }

    #[tokio::test]
    async fn revision_tracks_active_profiles() {
        let (store, _dir) = store().await;
        let initial = store.revision().await.unwrap();
        let bytes = build_dcp(Some("FUJIFILM X-T5"));
        let meta = store.import(None, None, false, &bytes).await.unwrap();
        let imported = store.revision().await.unwrap();
        if initial == imported {
            panic!("profile import did not change revision");
        }

        store.soft_delete(&meta.id).await.unwrap();
        let deleted = store.revision().await.unwrap();
        if deleted != initial {
            panic!("profile deletion did not restore revision");
        }
    }

    #[tokio::test]
    async fn every_bundled_profile_matches_on_auto() {
        let (store, _dir) = store().await;
        let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/dcp");
        store.import_bundled(&assets).await.unwrap();
        let manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(assets.join("manifest.json")).unwrap()).unwrap();
        let mut unmatched = Vec::new();
        for profile in manifest["profiles"].as_array().unwrap() {
            let Some(model) = profile["camera_model"].as_str() else {
                continue;
            };
            if store.match_camera_meta(model).await.unwrap().is_none() {
                unmatched.push(model.to_string());
            }
            let normalized = normalize_model(model);
            let core = strip_make(&normalized).to_string();
            if store.match_camera_meta(&core).await.unwrap().is_none() {
                unmatched.push(format!("{model} (as {core})"));
            }
        }
        assert!(unmatched.is_empty(), "no auto match for: {unmatched:#?}");
    }

    #[test]
    fn bundled_manifest_matches_profile_files() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/dcp");
        let manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(dir.join("manifest.json")).unwrap()).unwrap();
        let profiles = manifest["profiles"].as_array().unwrap();
        let listed: BTreeSet<String> = profiles
            .iter()
            .map(|profile| profile["file"].as_str().unwrap().to_string())
            .collect();
        let files: BTreeSet<String> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let path = entry.path();
                (path.extension().and_then(|ext| ext.to_str()) == Some("dcp"))
                    .then(|| entry.file_name().to_string_lossy().to_string())
            })
            .collect();
        assert_eq!(manifest["count"].as_u64(), Some(files.len() as u64));
        assert_eq!(listed, files);
        assert_eq!(
            manifest["source_revision"].as_str(),
            Some("039b9b89d43315be6b42e8fbb33b8cfb39edd4bf")
        );
    }
}
