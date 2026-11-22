use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};
use url::Url;

use crate::services::immich_capabilities::ImmichCapabilities;

mod geocode;
mod photon;

pub use geocode::{GeocodeError, Geocoder};
pub use photon::Place;

const MAX_URL_LEN: usize = 2048;
pub const DEFAULT_GEOCODER: &str = "https://photon.komoot.io";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapSettings {
    #[serde(rename = "styleLight")]
    pub style_light: Option<String>,
    #[serde(rename = "styleDark")]
    pub style_dark: Option<String>,
    #[serde(rename = "geocoderUrl")]
    pub geocoder_url: String,
}

impl Default for MapSettings {
    fn default() -> Self {
        Self {
            style_light: None,
            style_dark: None,
            geocoder_url: DEFAULT_GEOCODER.into(),
        }
    }
}

fn web_url(raw: &str, label: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.len() > MAX_URL_LEN {
        return Err(format!("{label} is longer than {MAX_URL_LEN} characters"));
    }
    let url = Url::parse(trimmed).map_err(|_| format!("{label} must be a web address"))?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(format!("{label} must be an http or https address"));
    }
    Ok(trimmed.trim_end_matches('/').to_string())
}

impl MapSettings {
    pub fn validated(self) -> Result<Self, String> {
        let style_light = self
            .style_light
            .filter(|url| !url.trim().is_empty())
            .map(|url| web_url(&url, "light style URL"))
            .transpose()?;
        let style_dark = self
            .style_dark
            .filter(|url| !url.trim().is_empty())
            .map(|url| web_url(&url, "dark style URL"))
            .transpose()?;
        if style_light.is_some() != style_dark.is_some() {
            return Err("set both style URLs, or neither to follow Immich".into());
        }
        let geocoder_url = if self.geocoder_url.trim().is_empty() {
            String::new()
        } else {
            web_url(&self.geocoder_url, "place search URL")?
        };
        Ok(Self {
            style_light,
            style_dark,
            geocoder_url,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StyleSource {
    Custom,
    Immich,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapStyles {
    pub light: Option<String>,
    pub dark: Option<String>,
    pub source: StyleSource,
}

pub fn resolve_styles(settings: &MapSettings, caps: &ImmichCapabilities) -> MapStyles {
    if let (Some(light), Some(dark)) = (&settings.style_light, &settings.style_dark) {
        return MapStyles {
            light: Some(light.clone()),
            dark: Some(dark.clone()),
            source: StyleSource::Custom,
        };
    }
    let light = caps.map_light_style.clone();
    let dark = caps.map_dark_style.clone();
    let source = if light.is_some() || dark.is_some() {
        StyleSource::Immich
    } else {
        StyleSource::None
    };
    MapStyles {
        light: light.clone().or_else(|| dark.clone()),
        dark: dark.or(light),
        source,
    }
}

#[derive(Clone)]
pub struct MapService {
    settings: Arc<RwLock<MapSettings>>,
    pub geocoder: Geocoder,
}

impl MapService {
    pub fn new(settings: MapSettings) -> Self {
        Self {
            settings: Arc::new(RwLock::new(settings)),
            geocoder: Geocoder::new(),
        }
    }

    pub fn settings(&self) -> MapSettings {
        self.settings
            .read()
            .map(|settings| settings.clone())
            .unwrap_or_default()
    }

    pub fn set_settings(&self, settings: MapSettings) {
        if let Ok(mut guard) = self.settings.write() {
            *guard = settings;
        }
    }
}

#[cfg(test)]
mod tests;
