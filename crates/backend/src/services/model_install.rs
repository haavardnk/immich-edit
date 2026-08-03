use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use segment::CatalogEntry;
use tokio::sync::Semaphore;

use crate::services::model_download::{DownloadError, fetch_catalog_aux, fetch_catalog_model};
use crate::services::model_store::{ModelStore, ModelStoreError};

pub struct InstallProgress {
    pub total: u64,
    pub received: u64,
    pub error: Option<String>,
}

struct Install {
    total: u64,
    received: Arc<AtomicU64>,
    error: Option<String>,
}

#[derive(Clone)]
pub struct ModelInstaller {
    store: ModelStore,
    installs: Arc<Mutex<HashMap<&'static str, Install>>>,
    permits: Arc<Semaphore>,
}

impl ModelInstaller {
    pub fn new(store: ModelStore) -> Self {
        Self {
            store,
            installs: Arc::new(Mutex::new(HashMap::new())),
            permits: Arc::new(Semaphore::new(1)),
        }
    }

    pub fn start(&self, entry: &'static CatalogEntry) {
        let received = {
            let mut installs = self.installs.lock().unwrap();
            if installs.get(entry.id).is_some_and(|i| i.error.is_none()) {
                return;
            }
            let received = Arc::new(AtomicU64::new(0));
            installs.insert(
                entry.id,
                Install {
                    total: entry.total_bytes(),
                    received: received.clone(),
                    error: None,
                },
            );
            received
        };

        let installer = self.clone();
        tokio::spawn(async move { installer.run(entry, received).await });
    }

    pub fn snapshot(&self) -> HashMap<&'static str, InstallProgress> {
        self.installs
            .lock()
            .unwrap()
            .iter()
            .map(|(id, install)| {
                (
                    *id,
                    InstallProgress {
                        total: install.total,
                        received: install.received.load(Ordering::Relaxed),
                        error: install.error.clone(),
                    },
                )
            })
            .collect()
    }

    async fn run(&self, entry: &'static CatalogEntry, received: Arc<AtomicU64>) {
        let Ok(_permit) = self.permits.acquire().await else {
            return;
        };
        let dir = self.store.dir().to_path_buf();

        let result = match fetch_catalog_model(entry, &dir, &received).await {
            Err(e) => Err(download_message(&e)),
            Ok(model) => match fetch_catalog_aux(entry, &dir, &received).await {
                Err(e) => Err(download_message(&e)),
                Ok(aux) => self
                    .store
                    .install_downloaded(entry, model, aux)
                    .await
                    .map_err(|e| {
                        tracing::error!(model = entry.id, error = %e, "model install rejected");
                        store_message(&e)
                    }),
            },
        };

        let mut installs = self.installs.lock().unwrap();
        match result {
            Ok(meta) => {
                tracing::info!(model = entry.id, size = meta.size, "model installed");
                installs.remove(entry.id);
            }
            Err(message) => {
                if let Some(install) = installs.get_mut(entry.id) {
                    install.error = Some(message);
                }
            }
        }
    }
}

fn download_message(err: &DownloadError) -> String {
    match err {
        DownloadError::Status(status) => format!("download server returned {status}"),
        DownloadError::Http(_) => "download was interrupted".into(),
        DownloadError::TooLarge => "download exceeded the declared size".into(),
        DownloadError::InsecureUrl => "model url is not https".into(),
        DownloadError::Io(_) => "could not write the model to disk".into(),
    }
}

fn store_message(err: &ModelStoreError) -> String {
    match err {
        ModelStoreError::Checksum { .. } => "download did not match the published checksum".into(),
        ModelStoreError::Invalid(message) => message.clone(),
        _ => "could not save the model".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::edits_store::EditsStore;
    use segment::catalog;

    async fn installer() -> (ModelInstaller, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let edits = EditsStore::migrated_memory().await.unwrap();
        let store = ModelStore::new(edits.pool(), dir.path()).unwrap();
        (ModelInstaller::new(store), dir)
    }

    #[tokio::test]
    async fn tracks_combined_total_and_surfaces_failures() {
        let (installer, _dir) = installer().await;
        let mut entry = catalog::find("sam2_tiny").unwrap().clone();
        entry.url = "http://example.invalid/encoder.onnx";
        if let Some(aux) = &mut entry.aux {
            aux.url = "http://example.invalid/decoder.onnx";
        }
        let entry = Box::leak(Box::new(entry));

        assert!(installer.snapshot().is_empty());
        installer.start(entry);
        assert_eq!(
            installer.snapshot().get(entry.id).unwrap().total,
            entry.size_bytes + entry.aux.as_ref().unwrap().size_bytes
        );

        let mut error = None;
        for _ in 0..100 {
            tokio::task::yield_now().await;
            error = installer
                .snapshot()
                .get(entry.id)
                .and_then(|p| p.error.clone());
            if error.is_some() {
                break;
            }
        }
        assert_eq!(error.as_deref(), Some("model url is not https"));
    }
}
