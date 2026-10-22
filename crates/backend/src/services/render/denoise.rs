use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;

use raw_pipeline::ai_denoise::DenoisePlane;
use raw_pipeline::edits::Edits;
use raw_pipeline::frame::RawFrame;
use uuid::Uuid;

use crate::services::denoise_store::{DenoiseStore, PlaneKey};
#[cfg(feature = "ml")]
use crate::services::model_store::ModelStore;

use super::RenderIdentity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaneState {
    Off,
    Pending,
    Ready(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetRevision {
    base: String,
    plane: PlaneState,
}

impl AssetRevision {
    pub fn new(base: String, plane: PlaneState) -> Self {
        Self { base, plane }
    }

    pub fn settled(&self) -> bool {
        self.plane != PlaneState::Pending
    }
}

impl fmt::Display for AssetRevision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.base)?;
        match self.plane {
            PlaneState::Off => Ok(()),
            PlaneState::Pending => f.write_str("-dn"),
            PlaneState::Ready(id) => write!(f, "-dn{id:x}"),
        }
    }
}

pub struct OwnerPlanes {
    identity: RenderIdentity,
    model: Option<String>,
    ready: HashSet<Uuid>,
}

impl OwnerPlanes {
    pub fn state(&self, source: Uuid, ai_denoise: bool) -> PlaneState {
        let Some(model) = self.model.as_ref().filter(|_| ai_denoise) else {
            return PlaneState::Off;
        };
        if !self.ready.contains(&source) {
            return PlaneState::Pending;
        }
        let key = PlaneKey::new(self.identity, source, model.clone());
        PlaneState::Ready(key.plane_id())
    }
}

#[derive(Clone)]
pub struct DenoiseInputs {
    store: DenoiseStore,
    #[cfg(feature = "ml")]
    models: ModelStore,
}

impl DenoiseInputs {
    #[cfg(feature = "ml")]
    pub fn new(store: DenoiseStore, models: ModelStore) -> Self {
        Self { store, models }
    }

    #[cfg(not(feature = "ml"))]
    pub fn new(store: DenoiseStore) -> Self {
        Self { store }
    }

    pub fn store(&self) -> &DenoiseStore {
        &self.store
    }

    #[cfg(feature = "ml")]
    pub async fn model(&self) -> Option<String> {
        match self.models.resolve_installed(ml::ModelKind::Denoise).await {
            Ok(model) => model,
            Err(error) => {
                tracing::warn!(%error, "denoise model lookup failed");
                None
            }
        }
    }

    #[cfg(not(feature = "ml"))]
    pub async fn model(&self) -> Option<String> {
        None
    }

    pub async fn key(
        &self,
        identity: RenderIdentity,
        source: Uuid,
        edits: &Edits,
    ) -> Option<PlaneKey> {
        if !edits.detail.clamped().ai_denoise_active() {
            return None;
        }
        Some(PlaneKey::new(identity, source, self.model().await?))
    }

    pub async fn plane(
        &self,
        identity: RenderIdentity,
        source: Uuid,
        edits: &Edits,
        frame: &RawFrame,
    ) -> Option<Arc<DenoisePlane>> {
        if !frame.meta.is_raw {
            return None;
        }
        let key = self.key(identity, source, edits).await?;
        let plane = self.store.get(&key).await?;
        if (plane.width, plane.height) == (frame.meta.width, frame.meta.height) {
            return Some(plane);
        }
        self.store.discard(&key).await;
        None
    }

    pub async fn state(&self, identity: RenderIdentity, source: Uuid, edits: &Edits) -> PlaneState {
        match self.key(identity, source, edits).await {
            Some(key) if self.store.contains(&key).await => PlaneState::Ready(key.plane_id()),
            Some(_) => PlaneState::Pending,
            None => PlaneState::Off,
        }
    }

    pub async fn owner_planes(&self, identity: RenderIdentity) -> OwnerPlanes {
        let model = self.model().await;
        let ready = match &model {
            Some(model) => {
                self.store
                    .ready_sources(identity.server_epoch, identity.owner, model)
                    .await
            }
            None => HashSet::new(),
        };
        OwnerPlanes {
            identity,
            model,
            ready,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_pending_plane_leaves_the_revision_unsettled() {
        let revision = |plane: PlaneState| AssetRevision::new("d0-l1".into(), plane);
        assert!(revision(PlaneState::Off).settled());
        assert!(revision(PlaneState::Ready(0xab)).settled());
        assert!(!revision(PlaneState::Pending).settled());
    }

    #[test]
    fn owner_planes_match_the_single_asset_state() {
        let identity = RenderIdentity {
            owner: Uuid::from_u128(1),
            server_epoch: 3,
        };
        let baked = Uuid::from_u128(2);
        let planes = OwnerPlanes {
            identity,
            model: Some("nind_utnet".into()),
            ready: HashSet::from([baked]),
        };
        let key = PlaneKey::new(identity, baked, "nind_utnet".into());
        assert_eq!(planes.state(baked, true), PlaneState::Ready(key.plane_id()));
        assert_eq!(planes.state(baked, false), PlaneState::Off);
        assert_eq!(planes.state(Uuid::from_u128(4), true), PlaneState::Pending);
        let no_model = OwnerPlanes {
            model: None,
            ..planes
        };
        assert_eq!(no_model.state(baked, true), PlaneState::Off);
    }
}
