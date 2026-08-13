use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{Semaphore, watch};

use crate::services::job_store::{JobItemRecord, JobRecord, JobStore};

pub type ItemOutcome = Result<serde_json::Value, JobItemError>;

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct JobItemError(String);

impl JobItemError {
    pub fn msg(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

macro_rules! item_error_from {
    ($($ty:path),+ $(,)?) => {
        $(impl From<$ty> for JobItemError {
            fn from(err: $ty) -> Self {
                Self(err.to_string())
            }
        })+
    };
}

item_error_from!(
    crate::error::AppError,
    crate::immich::ImmichError,
    crate::services::edits_store::EditsStoreError,
    crate::services::instance_store::InstanceStoreError,
    crate::services::job_store::JobStoreError,
    serde_json::Error,
    std::io::Error,
    url::ParseError,
);

pub trait JobExecutor: Send + Sync + 'static {
    fn execute(
        &self,
        job: JobRecord,
        item: JobItemRecord,
    ) -> Pin<Box<dyn Future<Output = ItemOutcome> + Send>>;
}

pub struct UnsupportedExecutor;

impl JobExecutor for UnsupportedExecutor {
    fn execute(
        &self,
        job: JobRecord,
        _item: JobItemRecord,
    ) -> Pin<Box<dyn Future<Output = ItemOutcome> + Send>> {
        Box::pin(async move {
            Err(JobItemError::msg(format!(
                "unsupported job kind: {}",
                job.kind
            )))
        })
    }
}

pub struct JobRunner {
    store: JobStore,
    executor: Arc<dyn JobExecutor>,
    concurrency: usize,
}

impl JobRunner {
    pub fn new(store: JobStore, executor: Arc<dyn JobExecutor>, concurrency: usize) -> Self {
        Self {
            store,
            executor,
            concurrency: concurrency.max(1),
        }
    }

    pub async fn run(self, mut shutdown: watch::Receiver<bool>) {
        if let Err(err) = self.store.requeue_running().await {
            tracing::error!(error = %err, "job runner failed to requeue running items");
        }
        let sem = Arc::new(Semaphore::new(self.concurrency));
        loop {
            if *shutdown.borrow() {
                break;
            }
            let Ok(permit) = sem.clone().acquire_owned().await else {
                break;
            };
            match self.store.claim_next_item().await {
                Ok(Some(item)) => {
                    let job = match self.store.get_job(item.job_id).await {
                        Ok(Some(job)) => job,
                        Ok(None) => {
                            drop(permit);
                            continue;
                        }
                        Err(err) => {
                            tracing::error!(error = %err, "job runner failed to load job");
                            drop(permit);
                            continue;
                        }
                    };
                    let store = self.store.clone();
                    let executor = self.executor.clone();
                    tokio::spawn(async move {
                        let item_id = item.id;
                        let outcome = executor.execute(job, item).await;
                        let res = match outcome {
                            Ok(value) => store.complete_item(item_id, &value).await,
                            Err(error) => store.fail_item(item_id, &error.to_string()).await,
                        };
                        if let Err(err) = res {
                            tracing::error!(error = %err, %item_id, "job runner failed to record item result");
                        }
                        drop(permit);
                    });
                }
                Ok(None) => {
                    drop(permit);
                    tokio::select! {
                        _ = shutdown.changed() => {}
                        _ = tokio::time::sleep(Duration::from_millis(500)) => {}
                    }
                }
                Err(err) => {
                    tracing::error!(error = %err, "job runner claim failed");
                    drop(permit);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
        tracing::info!("job runner stopped");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::edits_store::EditsStore;
    use crate::services::job_store::{NewJob, NewJobItem};
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct CountingExecutor {
        runs: Arc<AtomicUsize>,
    }

    impl JobExecutor for CountingExecutor {
        fn execute(
            &self,
            _job: JobRecord,
            item: JobItemRecord,
        ) -> Pin<Box<dyn Future<Output = ItemOutcome> + Send>> {
            let runs = self.runs.clone();
            Box::pin(async move {
                runs.fetch_add(1, Ordering::SeqCst);
                if item.asset_id == "bad" {
                    Err(JobItemError::msg("boom"))
                } else {
                    Ok(json!({"asset": item.asset_id}))
                }
            })
        }
    }

    #[tokio::test]
    async fn runner_processes_all_items() {
        let edits = EditsStore::migrated_memory().await.unwrap();
        let dir = tempfile::tempdir().unwrap();
        let crypto = Arc::new(
            crate::services::crypto::InstanceCrypto::load_or_create(
                &dir.path().join("instance.key"),
                false,
            )
            .unwrap(),
        );
        let store = JobStore::new(edits.pool(), crypto);
        let runs = Arc::new(AtomicUsize::new(0));
        let executor = Arc::new(CountingExecutor { runs: runs.clone() });

        let job = store
            .create_job(NewJob {
                owner: uuid::Uuid::nil(),
                server_epoch: 1,
                auth_session_id: uuid::Uuid::nil(),
                kind: "test",
                target: &json!(null),
                params: &json!(null),
                items: &[
                    NewJobItem {
                        asset_id: "ok".into(),
                        idempotency_key: None,
                    },
                    NewJobItem {
                        asset_id: "bad".into(),
                        idempotency_key: None,
                    },
                ],
                cred: b"test-key",
                auth_kind: crate::services::auth_store::AuthKind::ApiKey,
            })
            .await
            .unwrap();

        let (tx, rx) = watch::channel(false);
        let handle = tokio::spawn(JobRunner::new(store.clone(), executor, 2).run(rx));

        let mut done = false;
        for _ in 0..50 {
            let current = store.get_job(job.id).await.unwrap().unwrap();
            if current.completed + current.failed == current.total {
                done = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        let _ = tx.send(true);
        let _ = handle.await;

        assert!(done);
        assert_eq!(runs.load(Ordering::SeqCst), 2);
        let final_job = store.get_job(job.id).await.unwrap().unwrap();
        assert_eq!(final_job.completed, 1);
        assert_eq!(final_job.failed, 1);
    }
}
