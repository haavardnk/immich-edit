use std::path::PathBuf;
use std::str::FromStr;

use url::Url;

use super::ConfigError;

pub const REMOVED_KEYS: [(&str, &str); 5] = [
    ("CACHE_DIR", "DATA_DIR"),
    ("SEGMENT_RUNTIME", "ML_RUNTIME"),
    ("SEGMENT_MAX_EDGE", "ML_MAX_EDGE"),
    ("SEGMENT_MAX_CONCURRENCY", "ML_MAX_CONCURRENCY"),
    ("SEGMENT_IDLE_SECS", "ML_IDLE_SECS"),
];

pub fn reject_removed_keys(file: &toml::Table) -> Result<(), ConfigError> {
    for (key, replacement) in REMOVED_KEYS {
        let in_env = std::env::var(key).is_ok_and(|v| !v.is_empty());
        let in_file = file.contains_key(&key.to_ascii_lowercase());
        if in_env || in_file {
            return Err(ConfigError::RemovedKey { key, replacement });
        }
    }
    Ok(())
}

pub fn pick(env_key: &str, file_value: Option<String>) -> Option<String> {
    if let Ok(v) = std::env::var(env_key) {
        if !v.is_empty() {
            return Some(v);
        }
    }
    file_value.filter(|s| !s.is_empty())
}

pub fn parse_or<T>(env_key: &str, file_value: Option<T>, default: T) -> Result<T, ConfigError>
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

pub fn load_allowed_origins(file_value: Option<Vec<String>>) -> Result<Vec<String>, ConfigError> {
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

pub fn validate_allowed_origin(raw: &str) -> Result<String, ConfigError> {
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

pub fn invalid_origin(raw: &str) -> ConfigError {
    ConfigError::InvalidValue {
        key: "ALLOWED_ORIGINS".into(),
        value: raw.into(),
    }
}

pub fn ensure_dir_writable(dir: &PathBuf) -> Result<(), ConfigError> {
    std::fs::create_dir_all(dir).map_err(|source| ConfigError::DataDir {
        path: dir.clone(),
        source,
    })?;
    let probe = dir.join(".immich-edit-write-probe");
    std::fs::write(&probe, b"ok").map_err(|source| ConfigError::DataDir {
        path: dir.clone(),
        source,
    })?;
    let _ = std::fs::remove_file(&probe);
    Ok(())
}
