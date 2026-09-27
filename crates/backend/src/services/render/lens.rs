use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use lru::LruCache;
use raw_pipeline::edits::LensEdits;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::immich::ImmichClient;
use crate::lens_profile::ProfileLensEdits;

use super::RenderIdentity;

const CACHE_CAP: NonZeroUsize = NonZeroUsize::new(4096).unwrap();

type LensKey = (RenderIdentity, Uuid);

#[derive(Clone)]
pub struct LensProfiles {
    cache: Arc<Mutex<LruCache<LensKey, Option<ProfileLensEdits>>>>,
    auto: Arc<AtomicBool>,
}

impl Default for LensProfiles {
    fn default() -> Self {
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(CACHE_CAP))),
            auto: Arc::new(AtomicBool::new(true)),
        }
    }
}

impl LensProfiles {
    pub fn auto(&self) -> bool {
        self.auto.load(Ordering::Relaxed)
    }

    pub fn set_auto(&self, enabled: bool) {
        self.auto.store(enabled, Ordering::Relaxed);
    }

    pub async fn resolve(
        &self,
        identity: RenderIdentity,
        immich: &ImmichClient,
        source: Uuid,
        lens: LensEdits,
    ) -> LensEdits {
        if lens.profile_enabled.is_some() || !self.auto() {
            return lens;
        }
        let key = (identity, source);
        if let Some(profile) = self.cache.lock().await.get(&key) {
            return crate::lens_profile::apply_auto(lens, profile.as_ref());
        }
        let profile = match immich.asset(source).await {
            Ok(asset) => asset
                .exif_info
                .as_ref()
                .and_then(|exif| crate::lens_profile::lookup(exif).edits),
            Err(e) => {
                tracing::warn!(error = %e, "lens auto-resolve: asset lookup failed");
                return lens;
            }
        };
        self.cache.lock().await.put(key, profile.clone());
        crate::lens_profile::apply_auto(lens, profile.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use url::Url;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;
    use crate::immich::client::ImmichAuth;

    async fn sony_asset(expected_lookups: u64) -> (MockServer, ImmichClient, Uuid) {
        let server = MockServer::start().await;
        let source = Uuid::new_v4();
        Mock::given(method("GET"))
            .and(path(format!("/api/assets/{source}")))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": source,
                "exifInfo": { "make": "SONY", "model": "ILCE-7M3", "lensModel": "FE 35mm F1.8" },
            })))
            .expect(expected_lookups)
            .mount(&server)
            .await;
        let immich = ImmichClient::with_auth(
            Url::parse(&server.uri()).unwrap(),
            ImmichAuth::ApiKey("key".into()),
            Duration::from_secs(5),
        )
        .unwrap();
        (server, immich, source)
    }

    fn identity() -> RenderIdentity {
        RenderIdentity {
            owner: Uuid::new_v4(),
            server_epoch: 1,
        }
    }

    #[tokio::test]
    async fn instance_opt_out_leaves_unset_profiles_off() {
        let (server, immich, source) = sony_asset(0).await;
        let profiles = LensProfiles::default();
        profiles.set_auto(false);
        let lens = profiles
            .resolve(identity(), &immich, source, LensEdits::default())
            .await;
        assert_eq!(lens.profile_enabled, None);
        assert_eq!(lens.k1, 0.0);
        server.verify().await;
    }

    #[tokio::test]
    async fn instance_default_applies_matched_profile() {
        let (server, immich, source) = sony_asset(1).await;
        let lens = LensProfiles::default()
            .resolve(identity(), &immich, source, LensEdits::default())
            .await;
        assert_eq!(lens.profile_enabled, Some(true));
        server.verify().await;
    }

    #[tokio::test]
    async fn profiles_are_cached_per_render_identity() {
        let (server, immich, source) = sony_asset(2).await;
        let profiles = LensProfiles::default();
        let first = identity();
        let second = RenderIdentity {
            server_epoch: 2,
            ..first
        };
        for identity in [first, first, second, second] {
            profiles
                .resolve(identity, &immich, source, LensEdits::default())
                .await;
        }
        server.verify().await;
    }
}
