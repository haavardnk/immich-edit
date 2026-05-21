#[derive(Debug, Clone)]
pub struct Config {
    pub immich_url: String,
    pub immich_api_key: String,
    pub bind_addr: String,
    pub cache_dir: String,
}

impl Config {
    pub fn load() -> Self {
        Self {
            immich_url: std::env::var("IMMICH_URL").unwrap_or_else(|_| "http://localhost:2283".into()),
            immich_api_key: std::env::var("IMMICH_API_KEY").unwrap_or_default(),
            bind_addr: std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".into()),
            cache_dir: std::env::var("CACHE_DIR").unwrap_or_else(|_| "./cache".into()),
        }
    }
}
