use std::path::PathBuf;
use std::str::FromStr;

use serde::Deserialize;
use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RendererMode {
    Auto,
    Cpu,
    Gpu,
}

impl FromStr for RendererMode {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "cpu" => Ok(Self::Cpu),
            "gpu" => Ok(Self::Gpu),
            other => Err(ConfigError::InvalidValue {
                key: "IMMICH_EDIT_RENDERER".into(),
                value: other.to_string(),
            }),
        }
    }
}

impl RendererMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Cpu => "cpu",
            Self::Gpu => "gpu",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub immich_url: Url,
    pub immich_api_key: String,
    pub bind_addr: String,
    pub cache_dir: PathBuf,
    pub preview_max_edge: u32,
    pub raw_frame_cache_mb: u64,
    pub linear_cache_mb: u64,
    pub render_max_concurrency: usize,
    pub renderer: RendererMode,
}

#[derive(Debug, Deserialize, Default)]
struct FileConfig {
    immich_url: Option<String>,
    immich_api_key: Option<String>,
    bind_addr: Option<String>,
    cache_dir: Option<String>,
    preview_max_edge: Option<u32>,
    raw_frame_cache_mb: Option<u64>,
    linear_cache_mb: Option<u64>,
    render_max_concurrency: Option<usize>,
    renderer: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("missing required config: {0}")]
    Missing(&'static str),
    #[error("invalid value for {key}: {value}")]
    InvalidValue { key: String, value: String },
    #[error("config file {path}: {source}")]
    File {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("config file parse: {0}")]
    Parse(#[from] toml::de::Error),
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let file = match std::env::var("IMMICH_EDIT_CONFIG").ok() {
            Some(path) => {
                let path = PathBuf::from(path);
                let text = std::fs::read_to_string(&path)
                    .map_err(|source| ConfigError::File { path, source })?;
                toml::from_str::<FileConfig>(&text)?
            }
            None => FileConfig::default(),
        };

        let immich_url_raw =
            pick("IMMICH_URL", file.immich_url).ok_or(ConfigError::Missing("IMMICH_URL"))?;
        let immich_url = Url::parse(&immich_url_raw).map_err(|_| ConfigError::InvalidValue {
            key: "IMMICH_URL".into(),
            value: redact_url_str(&immich_url_raw),
        })?;
        if !matches!(immich_url.scheme(), "http" | "https") {
            return Err(ConfigError::InvalidValue {
                key: "IMMICH_URL".into(),
                value: immich_url.scheme().into(),
            });
        }

        let immich_api_key = pick("IMMICH_API_KEY", file.immich_api_key)
            .ok_or(ConfigError::Missing("IMMICH_API_KEY"))?;
        if immich_api_key.trim().is_empty() {
            return Err(ConfigError::Missing("IMMICH_API_KEY"));
        }

        let bind_addr =
            pick("BIND_ADDR", file.bind_addr).unwrap_or_else(|| "0.0.0.0:3000".into());
        let cache_dir = PathBuf::from(
            pick("CACHE_DIR", file.cache_dir).unwrap_or_else(|| "./cache".into()),
        );

        let preview_max_edge = parse_or("PREVIEW_MAX_EDGE", file.preview_max_edge, 2048u32)?;
        if !(256..=8192).contains(&preview_max_edge) {
            return Err(ConfigError::InvalidValue {
                key: "PREVIEW_MAX_EDGE".into(),
                value: preview_max_edge.to_string(),
            });
        }

        let raw_frame_cache_mb = parse_or("RAW_FRAME_CACHE_MB", file.raw_frame_cache_mb, 1024u64)?;
        let linear_cache_mb = parse_or("LINEAR_CACHE_MB", file.linear_cache_mb, 512u64)?;
        let render_max_concurrency =
            parse_or("RENDER_MAX_CONCURRENCY", file.render_max_concurrency, 2usize)?;
        if render_max_concurrency == 0 {
            return Err(ConfigError::InvalidValue {
                key: "RENDER_MAX_CONCURRENCY".into(),
                value: "0".into(),
            });
        }

        let renderer = match pick("IMMICH_EDIT_RENDERER", file.renderer) {
            Some(s) => s.parse()?,
            None => RendererMode::Cpu,
        };

        Ok(Self {
            immich_url,
            immich_api_key,
            bind_addr,
            cache_dir,
            preview_max_edge,
            raw_frame_cache_mb,
            linear_cache_mb,
            render_max_concurrency,
            renderer,
        })
    }

    pub fn redacted(&self) -> RedactedConfig {
        RedactedConfig {
            immich_url: redact_url(&self.immich_url),
            immich_api_key_present: !self.immich_api_key.is_empty(),
            bind_addr: self.bind_addr.clone(),
            cache_dir: self.cache_dir.display().to_string(),
            preview_max_edge: self.preview_max_edge,
            raw_frame_cache_mb: self.raw_frame_cache_mb,
            linear_cache_mb: self.linear_cache_mb,
            render_max_concurrency: self.render_max_concurrency,
            renderer: self.renderer.as_str(),
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct RedactedConfig {
    pub immich_url: String,
    pub immich_api_key_present: bool,
    pub bind_addr: String,
    pub cache_dir: String,
    pub preview_max_edge: u32,
    pub raw_frame_cache_mb: u64,
    pub linear_cache_mb: u64,
    pub render_max_concurrency: usize,
    pub renderer: &'static str,
}

fn pick(env_key: &str, file_value: Option<String>) -> Option<String> {
    if let Ok(v) = std::env::var(env_key) {
        if !v.is_empty() {
            return Some(v);
        }
    }
    file_value.filter(|s| !s.is_empty())
}

fn parse_or<T>(env_key: &str, file_value: Option<T>, default: T) -> Result<T, ConfigError>
where
    T: FromStr + Copy,
{
    if let Ok(v) = std::env::var(env_key) {
        return v.parse::<T>().map_err(|_| ConfigError::InvalidValue {
            key: env_key.into(),
            value: v,
        });
    }
    Ok(file_value.unwrap_or(default))
}

fn redact_url(url: &Url) -> String {
    let mut out = url.clone();
    let _ = out.set_username("");
    let _ = out.set_password(None);
    out.to_string()
}

fn redact_url_str(raw: &str) -> String {
    Url::parse(raw)
        .map(|u| redact_url(&u))
        .unwrap_or_else(|_| "<invalid>".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clear_env() {
        for k in [
            "IMMICH_URL",
            "IMMICH_API_KEY",
            "BIND_ADDR",
            "CACHE_DIR",
            "PREVIEW_MAX_EDGE",
            "RAW_FRAME_CACHE_MB",
            "LINEAR_CACHE_MB",
            "RENDER_MAX_CONCURRENCY",
            "IMMICH_EDIT_RENDERER",
            "IMMICH_EDIT_CONFIG",
        ] {
            unsafe { std::env::remove_var(k) };
        }
    }

    fn lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    #[test]
    fn loads_env_defaults() {
        let _g = lock();
        clear_env();
        unsafe {
            std::env::set_var("IMMICH_URL", "http://example.local:2283");
            std::env::set_var("IMMICH_API_KEY", "secret-key");
        }
        let cfg = Config::load().unwrap();
        if cfg.immich_url.as_str() != "http://example.local:2283/" {
            panic!("url: {}", cfg.immich_url);
        }
        if cfg.preview_max_edge != 2048 {
            panic!("max_edge");
        }
        if cfg.renderer != RendererMode::Cpu {
            panic!("renderer");
        }
    }

    #[test]
    fn rejects_missing_url() {
        let _g = lock();
        clear_env();
        unsafe { std::env::set_var("IMMICH_API_KEY", "x") };
        let err = Config::load().unwrap_err();
        if !matches!(err, ConfigError::Missing("IMMICH_URL")) {
            panic!("err {err:?}");
        }
    }

    #[test]
    fn rejects_blank_key() {
        let _g = lock();
        clear_env();
        unsafe {
            std::env::set_var("IMMICH_URL", "http://x");
            std::env::set_var("IMMICH_API_KEY", "   ");
        }
        let err = Config::load().unwrap_err();
        if !matches!(err, ConfigError::Missing("IMMICH_API_KEY")) {
            panic!("err {err:?}");
        }
    }

    #[test]
    fn rejects_bad_scheme() {
        let _g = lock();
        clear_env();
        unsafe {
            std::env::set_var("IMMICH_URL", "ftp://x");
            std::env::set_var("IMMICH_API_KEY", "k");
        }
        let err = Config::load().unwrap_err();
        if !matches!(err, ConfigError::InvalidValue { .. }) {
            panic!("err {err:?}");
        }
    }

    #[test]
    fn redacts_key_and_credentials() {
        let _g = lock();
        clear_env();
        unsafe {
            std::env::set_var("IMMICH_URL", "http://user:pw@example.local");
            std::env::set_var("IMMICH_API_KEY", "supersecret");
        }
        let cfg = Config::load().unwrap();
        let red = cfg.redacted();
        if red.immich_url.contains("supersecret") {
            panic!("key leak");
        }
        if red.immich_url.contains("user") || red.immich_url.contains("pw") {
            panic!("creds leak: {}", red.immich_url);
        }
        if !red.immich_api_key_present {
            panic!("flag");
        }
        let json = serde_json::to_string(&red).unwrap();
        if json.contains("supersecret") {
            panic!("serialized leak");
        }
    }

    #[test]
    fn renderer_parses() {
        let _g = lock();
        clear_env();
        unsafe {
            std::env::set_var("IMMICH_URL", "http://x");
            std::env::set_var("IMMICH_API_KEY", "k");
            std::env::set_var("IMMICH_EDIT_RENDERER", "auto");
        }
        let cfg = Config::load().unwrap();
        if cfg.renderer != RendererMode::Auto {
            panic!("renderer");
        }
    }
}
