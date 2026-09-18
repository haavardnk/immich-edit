use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use url::Url;

use crate::immich::client::{ImmichAuth, ImmichClient};

const TTL: Duration = Duration::from_secs(60);
const FETCH_TIMEOUT: Duration = Duration::from_secs(10);
const DEFAULT_BUTTON_TEXT: &str = "Login with OAuth";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Providers {
    pub oauth: bool,
    pub password_login: bool,
    pub auto_launch: bool,
    pub button_text: String,
    pub degraded: bool,
}

impl Providers {
    fn unreachable() -> Self {
        Self {
            oauth: false,
            password_login: true,
            auto_launch: false,
            button_text: DEFAULT_BUTTON_TEXT.into(),
            degraded: true,
        }
    }
}

#[derive(Clone, Default)]
pub struct ProviderCache {
    inner: Arc<Mutex<HashMap<String, (Instant, Providers)>>>,
}

impl ProviderCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get(&self, base: &Url) -> Providers {
        let key = base.as_str().to_string();
        if let Some(hit) = self.cached(&key) {
            return hit;
        }
        let fresh = fetch(base).await;
        if !fresh.degraded
            && let Ok(mut guard) = self.inner.lock()
        {
            guard.insert(key, (Instant::now(), fresh.clone()));
        }
        fresh
    }

    fn cached(&self, key: &str) -> Option<Providers> {
        let guard = self.inner.lock().ok()?;
        let (at, providers) = guard.get(key)?;
        if at.elapsed() < TTL {
            Some(providers.clone())
        } else {
            None
        }
    }
}

async fn fetch(base: &Url) -> Providers {
    let Ok(client) = ImmichClient::with_auth(
        base.clone(),
        ImmichAuth::ApiKey(String::new()),
        FETCH_TIMEOUT,
    ) else {
        return Providers::unreachable();
    };
    let features = match client.server_features().await {
        Ok(f) => f,
        Err(e) => {
            tracing::warn!(error = %e, "could not read immich server features");
            return Providers::unreachable();
        }
    };
    let button_text = match client.server_config().await {
        Ok(c) if !c.oauth_button_text.trim().is_empty() => c.oauth_button_text,
        _ => DEFAULT_BUTTON_TEXT.into(),
    };
    Providers {
        oauth: features.oauth,
        password_login: features.password_login,
        auto_launch: features.oauth && features.oauth_auto_launch,
        button_text,
        degraded: false,
    }
}
