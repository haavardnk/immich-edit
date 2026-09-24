use std::num::NonZeroUsize;
use std::sync::Arc;

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
}

impl Default for LensProfiles {
    fn default() -> Self {
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(CACHE_CAP))),
        }
    }
}

impl LensProfiles {
    pub async fn resolve(
        &self,
        identity: RenderIdentity,
        immich: &ImmichClient,
        source: Uuid,
        lens: LensEdits,
    ) -> LensEdits {
        if lens.profile_enabled.is_some() {
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

    #[tokio::test]
    async fn profiles_are_cached_per_render_identity() {
        let server = MockServer::start().await;
        let source = Uuid::new_v4();
        Mock::given(method("GET"))
            .and(path(format!("/api/assets/{source}")))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": source,
                "exifInfo": { "make": "SONY", "model": "ILCE-7M3", "lensModel": "FE 35mm F1.8" },
            })))
            .expect(2)
            .mount(&server)
            .await;
        let immich = ImmichClient::with_auth(
            Url::parse(&server.uri()).unwrap(),
            ImmichAuth::ApiKey("key".into()),
            Duration::from_secs(5),
        )
        .unwrap();
        let profiles = LensProfiles::default();
        let first = RenderIdentity {
            owner: Uuid::new_v4(),
            server_epoch: 1,
        };
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
