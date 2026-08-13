use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;

mod env;

use env::{ensure_dir_writable, load_allowed_origins, parse_or, pick, reject_removed_keys};
use serde::Deserialize;

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
    pub bind_addr: String,
    pub bind_socket: SocketAddr,
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub preview_max_edge: u32,
    pub render_max_concurrency: usize,
    pub thumb_max_concurrency: usize,
    pub mask_cache_mb: u64,
    pub embedding_cache_mb: u64,
    pub raw_frame_cache_mb: u64,
    pub quality_frame_cache_mb: u64,
    pub gpu_texture_cache_mb: u64,
    pub renderer: RendererMode,
    pub database_url: String,
    pub allowed_origins: Vec<String>,
    pub max_body_mb: u64,
    pub request_timeout_secs: u64,
    pub original_timeout_secs: u64,
    pub export_timeout_secs: u64,
    pub ml_runtime: MlRuntimeMode,
    pub ml_max_edge: u32,
    pub ml_max_concurrency: usize,
    pub ml_idle_secs: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MlRuntimeMode {
    #[default]
    Auto,
    Gpu,
    Cpu,
    Off,
}

impl MlRuntimeMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Gpu => "gpu",
            Self::Cpu => "cpu",
            Self::Off => "off",
        }
    }
}

impl std::str::FromStr for MlRuntimeMode {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "gpu" => Ok(Self::Gpu),
            "cpu" => Ok(Self::Cpu),
            "off" | "disabled" => Ok(Self::Off),
            other => Err(ConfigError::InvalidValue {
                key: "ML_RUNTIME".into(),
                value: other.into(),
            }),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
struct FileConfig {
    bind_addr: Option<String>,
    data_dir: Option<String>,
    preview_max_edge: Option<u32>,
    render_max_concurrency: Option<usize>,
    thumb_max_concurrency: Option<usize>,
    mask_cache_mb: Option<u64>,
    embedding_cache_mb: Option<u64>,
    raw_frame_cache_mb: Option<u64>,
    quality_frame_cache_mb: Option<u64>,
    gpu_texture_cache_mb: Option<u64>,
    renderer: Option<String>,
    database_url: Option<String>,
    allowed_origins: Option<Vec<String>>,
    max_body_mb: Option<u64>,
    request_timeout_secs: Option<u64>,
    original_timeout_secs: Option<u64>,
    export_timeout_secs: Option<u64>,
    ml_runtime: Option<String>,
    ml_max_edge: Option<u32>,
    ml_max_concurrency: Option<usize>,
    ml_idle_secs: Option<u64>,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
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
    #[error("{key} was removed; use {replacement} instead")]
    RemovedKey {
        key: &'static str,
        replacement: &'static str,
    },
    #[error("data dir not writable: {path}: {source}")]
    DataDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let (file, file_table) = match std::env::var("IMMICH_EDIT_CONFIG").ok() {
            Some(path) => {
                let path = PathBuf::from(path);
                let text = std::fs::read_to_string(&path)
                    .map_err(|source| ConfigError::File { path, source })?;
                (
                    toml::from_str::<FileConfig>(&text)?,
                    toml::from_str::<toml::Table>(&text)?,
                )
            }
            None => (FileConfig::default(), toml::Table::new()),
        };

        reject_removed_keys(&file_table)?;

        let bind_addr = pick("BIND_ADDR", file.bind_addr).unwrap_or_else(|| "0.0.0.0:3000".into());
        let bind_socket: SocketAddr = bind_addr.parse().map_err(|_| ConfigError::InvalidValue {
            key: "BIND_ADDR".into(),
            value: bind_addr.clone(),
        })?;
        let data_dir = match pick("DATA_DIR", file.data_dir) {
            Some(dir) => PathBuf::from(dir),
            None => PathBuf::from("./data"),
        };
        let cache_dir = data_dir.join("cache");

        let preview_max_edge = parse_or("PREVIEW_MAX_EDGE", file.preview_max_edge, 65535u32)?;
        if !(256..=65535).contains(&preview_max_edge) {
            return Err(ConfigError::InvalidValue {
                key: "PREVIEW_MAX_EDGE".into(),
                value: preview_max_edge.to_string(),
            });
        }

        let cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(2);

        let render_max_concurrency = parse_or(
            "RENDER_MAX_CONCURRENCY",
            file.render_max_concurrency,
            (cores / 2).clamp(2, 4),
        )?;
        if render_max_concurrency == 0 {
            return Err(ConfigError::InvalidValue {
                key: "RENDER_MAX_CONCURRENCY".into(),
                value: "0".into(),
            });
        }

        let thumb_max_concurrency = parse_or(
            "THUMB_MAX_CONCURRENCY",
            file.thumb_max_concurrency,
            (cores / 4).clamp(2, 4),
        )?;
        if thumb_max_concurrency == 0 {
            return Err(ConfigError::InvalidValue {
                key: "THUMB_MAX_CONCURRENCY".into(),
                value: "0".into(),
            });
        }

        let mask_cache_mb = parse_or("MASK_CACHE_MB", file.mask_cache_mb, 512u64)?;
        if mask_cache_mb == 0 {
            return Err(ConfigError::InvalidValue {
                key: "MASK_CACHE_MB".into(),
                value: "0".into(),
            });
        }

        let embedding_cache_mb = parse_or("EMBEDDING_CACHE_MB", file.embedding_cache_mb, 2048u64)?;
        if embedding_cache_mb == 0 {
            return Err(ConfigError::InvalidValue {
                key: "EMBEDDING_CACHE_MB".into(),
                value: "0".into(),
            });
        }

        let raw_frame_cache_mb = parse_or(
            "RAW_FRAME_CACHE_MB",
            file.raw_frame_cache_mb,
            (render_max_concurrency as u64 * 256).max(512),
        )?;
        if !(64..=16384).contains(&raw_frame_cache_mb) {
            return Err(ConfigError::InvalidValue {
                key: "RAW_FRAME_CACHE_MB".into(),
                value: raw_frame_cache_mb.to_string(),
            });
        }

        let quality_frame_cache_mb = parse_or(
            "QUALITY_FRAME_CACHE_MB",
            file.quality_frame_cache_mb,
            512u64,
        )?;
        if !(64..=16384).contains(&quality_frame_cache_mb) {
            return Err(ConfigError::InvalidValue {
                key: "QUALITY_FRAME_CACHE_MB".into(),
                value: quality_frame_cache_mb.to_string(),
            });
        }

        let gpu_texture_cache_mb =
            parse_or("GPU_TEXTURE_CACHE_MB", file.gpu_texture_cache_mb, 512u64)?;
        if !(64..=16384).contains(&gpu_texture_cache_mb) {
            return Err(ConfigError::InvalidValue {
                key: "GPU_TEXTURE_CACHE_MB".into(),
                value: gpu_texture_cache_mb.to_string(),
            });
        }

        let renderer = match pick("IMMICH_EDIT_RENDERER", file.renderer) {
            Some(s) => s.parse()?,
            None => RendererMode::Auto,
        };

        let database_url = pick("DATABASE_URL", file.database_url).unwrap_or_else(|| {
            let mut p = data_dir.clone();
            p.push("immich-edit.db");
            format!("sqlite://{}?mode=rwc", p.display())
        });

        let allowed_origins = load_allowed_origins(file.allowed_origins)?;

        let max_body_mb = parse_or("MAX_BODY_MB", file.max_body_mb, 128u64)?;
        if max_body_mb == 0 {
            return Err(ConfigError::InvalidValue {
                key: "MAX_BODY_MB".into(),
                value: "0".into(),
            });
        }
        let request_timeout_secs =
            parse_or("REQUEST_TIMEOUT_SECS", file.request_timeout_secs, 60u64)?;
        if request_timeout_secs == 0 {
            return Err(ConfigError::InvalidValue {
                key: "REQUEST_TIMEOUT_SECS".into(),
                value: "0".into(),
            });
        }
        let original_timeout_secs =
            parse_or("ORIGINAL_TIMEOUT_SECS", file.original_timeout_secs, 120u64)?;
        let export_timeout_secs =
            parse_or("EXPORT_TIMEOUT_SECS", file.export_timeout_secs, 300u64)?;

        let ml_runtime = match pick("ML_RUNTIME", file.ml_runtime) {
            Some(s) => s.parse()?,
            None => MlRuntimeMode::Auto,
        };
        let ml_max_edge = parse_or("ML_MAX_EDGE", file.ml_max_edge, 2048u32)?;
        if !(256..=8192).contains(&ml_max_edge) {
            return Err(ConfigError::InvalidValue {
                key: "ML_MAX_EDGE".into(),
                value: ml_max_edge.to_string(),
            });
        }
        let ml_max_concurrency = parse_or("ML_MAX_CONCURRENCY", file.ml_max_concurrency, 1usize)?;
        if ml_max_concurrency == 0 {
            return Err(ConfigError::InvalidValue {
                key: "ML_MAX_CONCURRENCY".into(),
                value: "0".into(),
            });
        }
        let ml_idle_secs = parse_or("ML_IDLE_SECS", file.ml_idle_secs, 60u64)?;

        ensure_dir_writable(&data_dir)?;

        Ok(Self {
            bind_addr,
            bind_socket,
            data_dir,
            cache_dir,
            preview_max_edge,
            render_max_concurrency,
            thumb_max_concurrency,
            mask_cache_mb,
            embedding_cache_mb,
            raw_frame_cache_mb,
            quality_frame_cache_mb,
            gpu_texture_cache_mb,
            renderer,
            database_url,
            allowed_origins,
            max_body_mb,
            request_timeout_secs,
            original_timeout_secs,
            export_timeout_secs,
            ml_runtime,
            ml_max_edge,
            ml_max_concurrency,
            ml_idle_secs,
        })
    }

    pub fn redacted(&self) -> RedactedConfig {
        RedactedConfig {
            bind_addr: self.bind_addr.clone(),
            data_dir: self.data_dir.display().to_string(),
            cache_dir: self.cache_dir.display().to_string(),
            preview_max_edge: self.preview_max_edge,
            render_max_concurrency: self.render_max_concurrency,
            thumb_max_concurrency: self.thumb_max_concurrency,
            mask_cache_mb: self.mask_cache_mb,
            embedding_cache_mb: self.embedding_cache_mb,
            raw_frame_cache_mb: self.raw_frame_cache_mb,
            quality_frame_cache_mb: self.quality_frame_cache_mb,
            gpu_texture_cache_mb: self.gpu_texture_cache_mb,
            renderer: self.renderer.as_str(),
            allowed_origins: self.allowed_origins.clone(),
            max_body_mb: self.max_body_mb,
            request_timeout_secs: self.request_timeout_secs,
            original_timeout_secs: self.original_timeout_secs,
            export_timeout_secs: self.export_timeout_secs,
            ml_runtime: self.ml_runtime.as_str(),
            ml_max_edge: self.ml_max_edge,
            ml_max_concurrency: self.ml_max_concurrency,
            ml_idle_secs: self.ml_idle_secs,
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct RedactedConfig {
    pub bind_addr: String,
    pub data_dir: String,
    pub cache_dir: String,
    pub preview_max_edge: u32,
    pub render_max_concurrency: usize,
    pub thumb_max_concurrency: usize,
    pub mask_cache_mb: u64,
    pub embedding_cache_mb: u64,
    pub raw_frame_cache_mb: u64,
    pub quality_frame_cache_mb: u64,
    pub gpu_texture_cache_mb: u64,
    pub renderer: &'static str,
    pub allowed_origins: Vec<String>,
    pub max_body_mb: u64,
    pub request_timeout_secs: u64,
    pub original_timeout_secs: u64,
    pub export_timeout_secs: u64,
    pub ml_runtime: &'static str,
    pub ml_max_edge: u32,
    pub ml_max_concurrency: usize,
    pub ml_idle_secs: u64,
}

#[cfg(test)]
mod tests {
    use super::env::REMOVED_KEYS;
    use super::*;

    fn clear_env() {
        for k in [
            "BIND_ADDR",
            "DATA_DIR",
            "CACHE_DIR",
            "SEGMENT_RUNTIME",
            "SEGMENT_MAX_EDGE",
            "SEGMENT_MAX_CONCURRENCY",
            "SEGMENT_IDLE_SECS",
            "PREVIEW_MAX_EDGE",
            "RAW_FRAME_CACHE_MB",
            "QUALITY_FRAME_CACHE_MB",
            "GPU_TEXTURE_CACHE_MB",
            "RENDER_MAX_CONCURRENCY",
            "THUMB_MAX_CONCURRENCY",
            "MASK_CACHE_MB",
            "IMMICH_EDIT_RENDERER",
            "IMMICH_EDIT_CONFIG",
            "ALLOWED_ORIGINS",
            "MAX_BODY_MB",
            "REQUEST_TIMEOUT_SECS",
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
            std::env::set_var("BIND_ADDR", "127.0.0.1:0");
        }
        let cfg = Config::load().unwrap();
        if cfg.preview_max_edge != 65535 {
            panic!("max_edge");
        }
        if cfg.raw_frame_cache_mb != (cfg.render_max_concurrency as u64 * 256).max(512) {
            panic!("raw_frame_cache_mb");
        }
        if !(2..=4).contains(&cfg.render_max_concurrency) {
            panic!("render_max_concurrency");
        }
        if !(2..=4).contains(&cfg.thumb_max_concurrency) {
            panic!("thumb_max_concurrency");
        }
        if cfg.mask_cache_mb != 512 {
            panic!("mask_cache_mb");
        }
        if cfg.embedding_cache_mb != 2048 {
            panic!("embedding_cache_mb");
        }
        if cfg.quality_frame_cache_mb != 512 {
            panic!("quality_frame_cache_mb");
        }
        if cfg.gpu_texture_cache_mb != 512 {
            panic!("gpu_texture_cache_mb");
        }
        if cfg.renderer != RendererMode::Auto {
            panic!("renderer");
        }
    }

    #[test]
    fn rejects_removed_env_keys() {
        for (key, replacement) in REMOVED_KEYS {
            let _g = lock();
            clear_env();
            unsafe {
                std::env::set_var("BIND_ADDR", "127.0.0.1:0");
                std::env::set_var(key, "whatever");
            }
            match Config::load() {
                Err(ConfigError::RemovedKey { key: k, .. }) if k == key => {}
                other => panic!("{key} should be rejected in favour of {replacement}: {other:?}"),
            }
            unsafe { std::env::remove_var(key) };
        }
    }

    #[test]
    fn renderer_parses() {
        let _g = lock();
        clear_env();
        unsafe {
            std::env::set_var("BIND_ADDR", "127.0.0.1:0");
            std::env::set_var("IMMICH_EDIT_RENDERER", "auto");
        }
        let cfg = Config::load().unwrap();
        if cfg.renderer != RendererMode::Auto {
            panic!("renderer");
        }
    }

    #[test]
    fn parses_allowed_origins_csv() {
        let _g = lock();
        clear_env();
        unsafe {
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
