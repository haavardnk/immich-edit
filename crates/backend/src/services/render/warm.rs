use std::sync::Arc;
use std::sync::Mutex as SyncMutex;
use std::sync::atomic::{AtomicBool, Ordering};

use uuid::Uuid;

use crate::immich::ImmichClient;

use super::frames::FrameStore;
use super::{RenderIdentity, decode_blocking};

pub const WARM_NEIGHBOURS: usize = 2;
const WARM_SLOTS: u64 = WARM_NEIGHBOURS as u64 + 1;

pub(super) struct WarmJob {
    pub identity: RenderIdentity,
    pub immich: ImmichClient,
    pub sources: Vec<Uuid>,
}

#[derive(Clone, Default)]
pub(super) struct FrameWarmer {
    pending: Arc<SyncMutex<Option<WarmJob>>>,
    running: Arc<AtomicBool>,
}

impl FrameWarmer {
    pub async fn submit(&self, frames: &FrameStore, job: WarmJob) -> bool {
        if !frames.holds_frames_like_largest(WARM_SLOTS).await {
            return false;
        }
        *self.pending.lock().unwrap() = Some(job);
        if self.running.swap(true, Ordering::AcqRel) {
            return true;
        }
        let warmer = self.clone();
        let frames = frames.clone();
        tokio::spawn(async move { warmer.drain(&frames).await });
        true
    }

    async fn drain(&self, frames: &FrameStore) {
        loop {
            while let Some(job) = self.take() {
                self.run(frames, job).await;
            }
            self.running.store(false, Ordering::Release);
            if !self.has_pending() || self.running.swap(true, Ordering::AcqRel) {
                return;
            }
        }
    }

    async fn run(&self, frames: &FrameStore, job: WarmJob) {
        for source in job.sources {
            if self.has_pending() || !frames.holds_frames_like_largest(WARM_SLOTS).await {
                return;
            }
            let loaded = frames
                .get_or_load(job.identity, &job.immich, source, decode_blocking)
                .await;
            if let Err(e) = loaded {
                tracing::debug!(%source, error = %e, "frame warm failed");
            }
        }
    }

    fn take(&self) -> Option<WarmJob> {
        self.pending.lock().unwrap().take()
    }

    fn has_pending(&self) -> bool {
        self.pending.lock().unwrap().is_some()
    }
}
