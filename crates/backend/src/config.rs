use std::net::SocketAddr;
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
    pub bind_socket: SocketAddr,
    pub cache_dir: PathBuf,
    pub preview_max_edge: u32,
    pub render_max_concurrency: usize,
    pub mask_cache_mb: u64,
    pub renderer: RendererMode,
    pub database_url: String,
    pub auth_token: Option<String>,
    pub allowed_origins: Vec<String>,
    pub debug_endpoints: bool,
    pub max_body_mb: u64,
    pub original_timeout_secs: u64,
    pub export_timeout_secs: u64,
    pub insecure: bool,
}

#[derive(Debug, Deserialize, Default)]
struct FileConfig {
    immich_url: Option<String>,
    immich_api_key: Option<String>,
    bind_addr: Option<String>,
    cache_dir: Option<String>,
    preview_max_edge: Option<u32>,
    render_max_concurrency: Option<usize>,
    mask_cache_mb: Option<u64>,
    renderer: Option<String>,
    database_url: Option<String>,
    auth_token: Option<String>,
    allowed_origins: Option<Vec<String>>,
    debug_endpoints: Option<bool>,
    max_body_mb: Option<u64>,
    original_timeout_secs: Option<u64>,
    export_timeout_secs: Option<u64>,
    insecure: Option<bool>,
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
    #[error("cache dir not writable: {path}: {source}")]
    CacheDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(
        "AUTH_TOKEN unset and BIND_ADDR is non-loopback ({addr}); set AUTH_TOKEN, bind to 127.0.0.1, or set IMMICH_EDIT_INSECURE=1"
    )]
    InsecureBind { addr: String },
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

        let bind_addr = pick("BIND_ADDR", file.bind_addr).unwrap_or_else(|| "0.0.0.0:3000".into());
        let bind_socket: SocketAddr = bind_addr.parse().map_err(|_| ConfigError::InvalidValue {
            key: "BIND_ADDR".into(),
            value: bind_addr.clone(),
        })?;
        let cache_dir =
            PathBuf::from(pick("CACHE_DIR", file.cache_dir).unwrap_or_else(|| "./cache".into()));

        let preview_max_edge = parse_or("PREVIEW_MAX_EDGE", file.preview_max_edge, 4096u32)?;
        if !(256..=8192).contains(&preview_max_edge) {
            return Err(ConfigError::InvalidValue {
                key: "PREVIEW_MAX_EDGE".into(),
                value: preview_max_edge.to_string(),
            });
        }

        let render_max_concurrency = parse_or(
            "RENDER_MAX_CONCURRENCY",
            file.render_max_concurrency,
            2usize,
        )?;
        if render_max_concurrency == 0 {
            return Err(ConfigError::InvalidValue {
                key: "RENDER_MAX_CONCURRENCY".into(),
                value: "0".into(),
            });
        }

        let mask_cache_mb = parse_or("MASK_CACHE_MB", file.mask_cache_mb, 1024u64)?;
        if mask_cache_mb == 0 {
            return Err(ConfigError::InvalidValue {
                key: "MASK_CACHE_MB".into(),
                value: "0".into(),
            });
        }

        let renderer = match pick("IMMICH_EDIT_RENDERER", file.renderer) {
            Some(s) => s.parse()?,
            None => RendererMode::Auto,
        };

        let database_url = pick("DATABASE_URL", file.database_url).unwrap_or_else(|| {
            let mut p = cache_dir.clone();
            p.push("immich-edit.db");
            format!("sqlite://{}?mode=rwc", p.display())
        });

        let auth_token = pick("AUTH_TOKEN", file.auth_token).and_then(|s| {
            let t = s.trim().to_string();
            if t.is_empty() { None } else { Some(t) }
        });

        let allowed_origins = load_allowed_origins(file.allowed_origins)?;

        let debug_endpoints =
            parse_bool("IMMICH_EDIT_DEBUG", file.debug_endpoints)?.unwrap_or(false);
        let insecure = parse_bool("IMMICH_EDIT_INSECURE", file.insecure)?.unwrap_or(false);

        let max_body_mb = parse_or("MAX_BODY_MB", file.max_body_mb, 128u64)?;
        if max_body_mb == 0 {
            return Err(ConfigError::InvalidValue {
                key: "MAX_BODY_MB".into(),
                value: "0".into(),
            });
        }
        let original_timeout_secs =
            parse_or("ORIGINAL_TIMEOUT_SECS", file.original_timeout_secs, 120u64)?;
        let export_timeout_secs =
            parse_or("EXPORT_TIMEOUT_SECS", file.export_timeout_secs, 300u64)?;

        if !is_loopback(&bind_socket) && auth_token.is_none() && !insecure {
            return Err(ConfigError::InsecureBind {
                addr: bind_addr.clone(),
            });
        }

        ensure_cache_writable(&cache_dir)?;

        Ok(Self {
            immich_url,
            immich_api_key,
            bind_addr,
            bind_socket,
            cache_dir,
            preview_max_edge,
            render_max_concurrency,
            mask_cache_mb,
            renderer,
            database_url,
            auth_token,
            allowed_origins,
            debug_endpoints,
            max_body_mb,
            original_timeout_secs,
            export_timeout_secs,
            insecure,
        })
    }

    pub fn redacted(&self) -> RedactedConfig {
        RedactedConfig {
            immich_url: redact_url(&self.immich_url),
            immich_api_key_present: !self.immich_api_key.is_empty(),
            bind_addr: self.bind_addr.clone(),
            cache_dir: self.cache_dir.display().to_string(),
            preview_max_edge: self.preview_max_edge,
            render_max_concurrency: self.render_max_concurrency,
            mask_cache_mb: self.mask_cache_mb,
            renderer: self.renderer.as_str(),
            auth_enabled: self.auth_token.is_some(),
            allowed_origins: self.allowed_origins.clone(),
            debug_endpoints: self.debug_endpoints,
            max_body_mb: self.max_body_mb,
            original_timeout_secs: self.original_timeout_secs,
            export_timeout_secs: self.export_timeout_secs,
            insecure: self.insecure,
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
    pub render_max_concurrency: usize,
    pub mask_cache_mb: u64,
    pub renderer: &'static str,
    pub auth_enabled: bool,
    pub allowed_origins: Vec<String>,
    pub debug_endpoints: bool,
    pub max_body_mb: u64,
    pub original_timeout_secs: u64,
    pub export_timeout_secs: u64,
    pub insecure: bool,
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

fn parse_bool(env_key: &str, file_value: Option<bool>) -> Result<Option<bool>, ConfigError> {
    if let Ok(v) = std::env::var(env_key) {
        let lowered = v.to_ascii_lowercase();
        return match lowered.as_str() {
            "1" | "true" | "yes" | "on" => Ok(Some(true)),
            "0" | "false" | "no" | "off" | "" => Ok(Some(false)),
            _ => Err(ConfigError::InvalidValue {
                key: env_key.into(),
                value: v,
            }),
        };
    }
    Ok(file_value)
}

fn load_allowed_origins(file_value: Option<Vec<String>>) -> Result<Vec<String>, ConfigError> {
    let raw = match std::env::var("ALLOWED_ORIGINS").ok() {
        Some(s) if !s.is_empty() => s.split(',').map(str::to_string).collect(),
        _ => file_value.unwrap_or_default(),
    };
    raw.iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(validate_allowed_origin)
        .collect()
}

fn validate_allowed_origin(raw: &str) -> Result<String, ConfigError> {
    let parsed = Url::parse(raw).map_err(|_| invalid_origin(raw))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(invalid_origin(raw));
    }
    if parsed.host_str().is_none() {
        return Err(invalid_origin(raw));
    }
    if parsed.path() != "/" || parsed.query().is_some() || parsed.fragment().is_some() {
        return Err(invalid_origin(raw));
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(invalid_origin(raw));
    }
    let origin = parsed.origin().ascii_serialization();
    if raw != origin {
        return Err(invalid_origin(raw));
    }
    Ok(origin)
}

fn invalid_origin(raw: &str) -> ConfigError {
    ConfigError::InvalidValue {
        key: "ALLOWED_ORIGINS".into(),
        value: raw.into(),
    }
}

fn is_loopback(addr: &SocketAddr) -> bool {
    addr.ip().is_loopback()
}

fn ensure_cache_writable(dir: &PathBuf) -> Result<(), ConfigError> {
    std::fs::create_dir_all(dir).map_err(|source| ConfigError::CacheDir {
        path: dir.clone(),
        source,
    })?;
    let probe = dir.join(".immich-edit-write-probe");
    std::fs::write(&probe, b"ok").map_err(|source| ConfigError::CacheDir {
        path: dir.clone(),
        source,
    })?;
    let _ = std::fs::remove_file(&probe);
    Ok(())
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
            "MASK_CACHE_MB",
            "IMMICH_EDIT_RENDERER",
            "IMMICH_EDIT_CONFIG",
            "AUTH_TOKEN",
            "ALLOWED_ORIGINS",
            "IMMICH_EDIT_DEBUG",
            "IMMICH_EDIT_INSECURE",
            "MAX_BODY_MB",
            "ORIGINAL_TIMEOUT_SECS",
            "EXPORT_TIMEOUT_SECS",
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
            std::env::set_var("BIND_ADDR", "127.0.0.1:0");
        }
        let cfg = Config::load().unwrap();
        if cfg.immich_url.as_str() != "http://example.local:2283/" {
            panic!("url: {}", cfg.immich_url);
        }
        if cfg.preview_max_edge != 4096 {
            panic!("max_edge");
        }
        if cfg.renderer != RendererMode::Auto {
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
            std::env::set_var("BIND_ADDR", "127.0.0.1:0");
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
            std::env::set_var("BIND_ADDR", "127.0.0.1:0");
            std::env::set_var("IMMICH_EDIT_RENDERER", "auto");
        }
        let cfg = Config::load().unwrap();
        if cfg.renderer != RendererMode::Auto {
            panic!("renderer");
        }
    }

    #[test]
    fn rejects_non_loopback_without_auth() {
        let _g = lock();
        clear_env();
        unsafe {
            std::env::set_var("IMMICH_URL", "http://x");
            std::env::set_var("IMMICH_API_KEY", "k");
            std::env::set_var("BIND_ADDR", "0.0.0.0:3000");
        }
        let err = Config::load().unwrap_err();
        if !matches!(err, ConfigError::InsecureBind { .. }) {
            panic!("err {err:?}");
        }
    }

    #[test]
    fn allows_non_loopback_with_auth() {
        let _g = lock();
        clear_env();
        unsafe {
            std::env::set_var("IMMICH_URL", "http://x");
            std::env::set_var("IMMICH_API_KEY", "k");
            std::env::set_var("BIND_ADDR", "0.0.0.0:3000");
            std::env::set_var("AUTH_TOKEN", "tok");
        }
        let cfg = Config::load().unwrap();
        if cfg.auth_token.as_deref() != Some("tok") {
            panic!("auth");
        }
    }

    #[test]
    fn allows_non_loopback_with_insecure_flag() {
        let _g = lock();
        clear_env();
        unsafe {
            std::env::set_var("IMMICH_URL", "http://x");
            std::env::set_var("IMMICH_API_KEY", "k");
            std::env::set_var("BIND_ADDR", "0.0.0.0:3000");
            std::env::set_var("IMMICH_EDIT_INSECURE", "1");
        }
        let cfg = Config::load().unwrap();
        if !cfg.insecure {
            panic!("flag");
        }
    }

    #[test]
    fn parses_allowed_origins_csv() {
        let _g = lock();
        clear_env();
        unsafe {
            std::env::set_var("IMMICH_URL", "http://x");
            std::env::set_var("IMMICH_API_KEY", "k");
            std::env::set_var("BIND_ADDR", "127.0.0.1:0");
            std::env::set_var("ALLOWED_ORIGINS", "http://a.local, https://b.local:8443");
        }
        let cfg = Config::load().unwrap();
        if cfg.allowed_origins
            != vec![
                "http://a.local".to_string(),
                "https://b.local:8443".to_string(),
            ]
        {
            panic!("origins {:?}", cfg.allowed_origins);
        }
    }

    #[test]
    fn rejects_malformed_allowed_origin() {
        let _g = lock();
        for bad in [
            "https://edit.example.com/api",
            "https://edit.example.com/",
            "edit.example.com",
        ] {
            clear_env();
            unsafe {
                std::env::set_var("IMMICH_URL", "http://x");
                std::env::set_var("IMMICH_API_KEY", "k");
                std::env::set_var("BIND_ADDR", "127.0.0.1:0");
                std::env::set_var("ALLOWED_ORIGINS", bad);
            }
            let err = Config::load().unwrap_err();
            if !matches!(err, ConfigError::InvalidValue { .. }) {
                panic!("expected invalid for {bad}, got {err:?}");
            }
        }
    }
}
