use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use futures_util::StreamExt;
use ml::CatalogEntry;
use sha2::{Digest, Sha256};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::services::model_store::MAX_MODEL_BYTES;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(20);
const TOTAL_TIMEOUT: Duration = Duration::from_secs(30 * 60);

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("http: {0}")]
    Http(String),
    #[error("upstream returned {0}")]
    Status(u16),
    #[error("model exceeds the declared size")]
    TooLarge,
    #[error("insecure model url")]
    InsecureUrl,
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct Downloaded {
    pub path: PathBuf,
    pub hash: String,
    pub len: u64,
}

impl Drop for Downloaded {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

pub async fn fetch_catalog_model(
    entry: &CatalogEntry,
    dir: &Path,
    progress: &AtomicU64,
) -> Result<Downloaded, DownloadError> {
    require_https(entry.url)?;
    download_to_file(entry.url, entry.size_bytes, dir, progress).await
}

pub async fn fetch_catalog_aux(
    entry: &CatalogEntry,
    dir: &Path,
    progress: &AtomicU64,
) -> Result<Option<Downloaded>, DownloadError> {
    let Some(aux) = &entry.aux else {
        return Ok(None);
    };
    require_https(aux.url)?;
    download_to_file(aux.url, aux.size_bytes, dir, progress)
        .await
        .map(Some)
}

fn require_https(url: &str) -> Result<(), DownloadError> {
    if url.starts_with("https://") {
        return Ok(());
    }
    tracing::warn!(url, "model download rejected: url is not https");
    Err(DownloadError::InsecureUrl)
}

async fn download_to_file(
    url: &str,
    size_bytes: u64,
    dir: &Path,
    progress: &AtomicU64,
) -> Result<Downloaded, DownloadError> {
    let limit = size_bytes
        .saturating_add(size_bytes / 10)
        .min(MAX_MODEL_BYTES);

    let client = reqwest::Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(TOTAL_TIMEOUT)
        .build()
        .map_err(|e| DownloadError::Http(e.to_string()))?;

    let response = client.get(url).send().await.map_err(|e| {
        tracing::warn!(url, error = %e, "model download failed to connect");
        DownloadError::Http(e.to_string())
    })?;
    let status = response.status().as_u16();
    if !response.status().is_success() {
        tracing::warn!(url, status, "model download rejected by upstream");
        return Err(DownloadError::Status(status));
    }
    if let Some(len) = response.content_length()
        && len > limit
    {
        tracing::warn!(url, len, limit, "model download exceeds declared size");
        return Err(DownloadError::TooLarge);
    }

    let mut out = Downloaded {
        path: dir.join(format!(".tmp-{}", Uuid::new_v4())),
        hash: String::new(),
        len: 0,
    };
    let mut file = fs::File::create(&out.path).await?;
    let mut hasher = Sha256::new();
    let mut received: u64 = 0;
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| {
            tracing::warn!(url, received, error = %e, "model download interrupted");
            DownloadError::Http(e.to_string())
        })?;
        received += chunk.len() as u64;
        if received > limit {
            tracing::warn!(url, received, limit, "model download exceeds declared size");
            return Err(DownloadError::TooLarge);
        }
        hasher.update(&chunk);
        file.write_all(&chunk).await?;
        progress.fetch_add(chunk.len() as u64, Ordering::Relaxed);
    }
    file.flush().await?;
    drop(file);

    out.hash = hex::encode(hasher.finalize());
    out.len = received;
    tracing::info!(url, received, "model download complete");
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ml::catalog;
    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn rejects_non_https_url() {
        let dir = tempfile::tempdir().unwrap();
        let mut entry = catalog::find("ormbg").unwrap().clone();
        entry.url = "http://example.invalid/model.onnx";
        let err = fetch_catalog_model(&entry, dir.path(), &AtomicU64::new(0))
            .await
            .unwrap_err();
        assert!(matches!(err, DownloadError::InsecureUrl));
    }

    #[tokio::test]
    async fn every_catalog_url_is_https() {
        for entry in catalog::CATALOG {
            assert!(entry.url.starts_with("https://"), "{}", entry.id);
            if let Some(aux) = &entry.aux {
                assert!(aux.url.starts_with("https://"), "{} aux", entry.id);
            }
        }
    }

    #[tokio::test]
    async fn streams_to_disk_and_hashes() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![7u8; 4096]))
            .mount(&server)
            .await;
        let dir = tempfile::tempdir().unwrap();
        let progress = AtomicU64::new(0);
        let got = download_to_file(&server.uri(), 4096, dir.path(), &progress)
            .await
            .unwrap();

        assert_eq!(got.len, 4096);
        assert_eq!(progress.load(Ordering::Relaxed), 4096);
        assert_eq!(got.hash, hex::encode(Sha256::digest([7u8; 4096])));
        assert_eq!(fs::read(&got.path).await.unwrap(), vec![7u8; 4096]);
    }

    #[tokio::test]
    async fn dropping_downloaded_removes_the_temp_file() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![1u8; 64]))
            .mount(&server)
            .await;
        let dir = tempfile::tempdir().unwrap();
        let path = download_to_file(&server.uri(), 64, dir.path(), &AtomicU64::new(0))
            .await
            .unwrap()
            .path
            .clone();

        assert!(!path.exists());
    }

    #[tokio::test]
    async fn surfaces_upstream_failures() {
        let cases = [
            (404u16, 64usize, 64u64, DownloadError::Status(404)),
            (200, 8192, 16, DownloadError::TooLarge),
        ];
        for (status, body_len, declared, want) in cases {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .respond_with(ResponseTemplate::new(status).set_body_bytes(vec![0u8; body_len]))
                .mount(&server)
                .await;
            let dir = tempfile::tempdir().unwrap();
            let err = download_to_file(&server.uri(), declared, dir.path(), &AtomicU64::new(0))
                .await
                .unwrap_err();

            assert_eq!(err.to_string(), want.to_string());
            let mut entries = fs::read_dir(dir.path()).await.unwrap();
            assert!(entries.next_entry().await.unwrap().is_none());
        }
    }
}
