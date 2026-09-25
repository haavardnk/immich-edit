use std::collections::HashMap;
use std::future::Future;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use ml::CatalogEntry;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;

use crate::services::model_download::{
    DownloadError, Downloaded, fetch_catalog_aux, fetch_catalog_model,
};
use crate::services::model_store::{ModelStore, ModelStoreError};

pub struct InstallProgress {
    pub total: u64,
    pub received: u64,
    pub error: Option<String>,
}

struct Install {
    generation: u64,
    total: u64,
    received: Arc<AtomicU64>,
    error: Option<String>,
    cancel: CancellationToken,
}

struct Started {
    generation: u64,
    received: Arc<AtomicU64>,
    cancel: CancellationToken,
}

type Fetched = (Downloaded, Option<Downloaded>);

#[derive(Clone)]
pub struct ModelInstaller {
    store: ModelStore,
    installs: Arc<Mutex<HashMap<&'static str, Install>>>,
    permits: Arc<Semaphore>,
    generations: Arc<AtomicU64>,
}

impl ModelInstaller {
    pub fn new(store: ModelStore) -> Self {
        Self {
            store,
            installs: Arc::new(Mutex::new(HashMap::new())),
            permits: Arc::new(Semaphore::new(1)),
            generations: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn start(&self, entry: &'static CatalogEntry) {
        let Some(started) = self.register(entry) else {
            return;
        };
        let installer = self.clone();
        tokio::spawn(async move {
            let dir = installer.store.dir().to_path_buf();
            let fetch = fetch_both(entry, &dir, &started.received);
            installer
                .run(entry, started.generation, started.cancel.clone(), fetch)
                .await;
        });
    }

    pub fn cancel(&self, id: &str) -> bool {
        let Some(install) = self.installs.lock().unwrap().remove(id) else {
            return false;
        };
        install.cancel.cancel();
        tracing::info!(model = id, "model install cancelled");
        true
    }

    fn register(&self, entry: &'static CatalogEntry) -> Option<Started> {
        let mut installs = self.installs.lock().unwrap();
        if installs.get(entry.id).is_some_and(|i| i.error.is_none()) {
            return None;
        }
        let started = Started {
            generation: self.generations.fetch_add(1, Ordering::Relaxed),
            received: Arc::new(AtomicU64::new(0)),
            cancel: CancellationToken::new(),
        };
        installs.insert(
            entry.id,
            Install {
                generation: started.generation,
                total: entry.total_bytes(),
                received: started.received.clone(),
                error: None,
                cancel: started.cancel.clone(),
            },
        );
        Some(started)
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

    async fn run(
        &self,
        entry: &'static CatalogEntry,
        generation: u64,
        cancel: CancellationToken,
        fetch: impl Future<Output = Result<Fetched, DownloadError>>,
    ) {
        let _permit = tokio::select! {
            () = cancel.cancelled() => return,
            permit = self.permits.acquire() => match permit {
                Ok(permit) => permit,
                Err(_) => return,
            },
        };
        let fetched = tokio::select! {
            () = cancel.cancelled() => return,
            fetched = fetch => fetched,
        };
        if cancel.is_cancelled() {
            return;
        }

        let result = match fetched {
            Err(e) => Err(download_message(&e)),
            Ok((model, aux)) => self
                .store
                .install_downloaded(entry, model, aux)
                .await
                .map_err(|e| {
                    tracing::error!(model = entry.id, error = %e, "model install rejected");
                    store_message(&e)
                }),
        };

        let mut installs = self.installs.lock().unwrap();
        if installs
            .get(entry.id)
            .is_none_or(|i| i.generation != generation)
        {
            return;
        }
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

async fn fetch_both(
    entry: &CatalogEntry,
    dir: &Path,
    received: &AtomicU64,
) -> Result<Fetched, DownloadError> {
    let model = fetch_catalog_model(entry, dir, received).await?;
    let aux = fetch_catalog_aux(entry, dir, received).await?;
    Ok((model, aux))
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
    use ml::catalog;

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

    #[tokio::test]
    async fn cancel_ends_a_stalled_or_queued_install() {
        for hold_permit in [false, true] {
            let (installer, _dir) = installer().await;
            let entry = catalog::find("ormbg").unwrap();
            let blocker = if hold_permit {
                Some(installer.permits.clone().acquire_owned().await.unwrap())
            } else {
                None
            };
            let started = installer.register(entry).unwrap();
            let task = tokio::spawn({
                let installer = installer.clone();
                async move {
                    installer
                        .run(
                            entry,
                            started.generation,
                            started.cancel,
                            std::future::pending(),
                        )
                        .await
                }
            });
            tokio::task::yield_now().await;

            assert!(installer.cancel(entry.id));
            tokio::time::timeout(std::time::Duration::from_secs(1), task)
                .await
                .unwrap_or_else(|_| panic!("install kept running (queued: {hold_permit})"))
                .unwrap();
            drop(blocker);
            assert!(installer.snapshot().is_empty());
            assert_eq!(installer.permits.available_permits(), 1);
            assert!(!installer.cancel(entry.id));
        }
    }

    #[tokio::test]
    async fn a_superseded_install_leaves_the_new_one_alone() {
        let (installer, _dir) = installer().await;
        let entry = catalog::find("ormbg").unwrap();
        let first = installer.register(entry).unwrap();
        installer.cancel(entry.id);
        installer.register(entry).unwrap();

        installer
            .run(
                entry,
                first.generation,
                CancellationToken::new(),
                std::future::ready(Err(DownloadError::Status(404))),
            )
            .await;

        let progress = installer.snapshot();
        let current = progress.get(entry.id).unwrap();
        assert_eq!(current.error, None);
    }
}
