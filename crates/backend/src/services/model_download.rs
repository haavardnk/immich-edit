use std::time::Duration;

use futures_util::StreamExt;
use segment::CatalogEntry;

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
}

pub async fn fetch_catalog_model(entry: &CatalogEntry) -> Result<Vec<u8>, DownloadError> {
    if !entry.url.starts_with("https://") {
        return Err(DownloadError::InsecureUrl);
    }
    let limit = entry
        .size_bytes
        .saturating_add(entry.size_bytes / 10)
        .min(MAX_MODEL_BYTES);

    let client = reqwest::Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(TOTAL_TIMEOUT)
        .build()
        .map_err(|e| DownloadError::Http(e.to_string()))?;

    let response = client
        .get(entry.url)
        .send()
        .await
        .map_err(|e| DownloadError::Http(e.to_string()))?;
    if !response.status().is_success() {
        return Err(DownloadError::Status(response.status().as_u16()));
    }
    if let Some(len) = response.content_length()
        && len > limit
    {
        return Err(DownloadError::TooLarge);
    }

    let mut buf: Vec<u8> = Vec::with_capacity(entry.size_bytes as usize);
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| DownloadError::Http(e.to_string()))?;
        if buf.len() as u64 + chunk.len() as u64 > limit {
            return Err(DownloadError::TooLarge);
        }
        buf.extend_from_slice(&chunk);
    }
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use segment::catalog;

    #[tokio::test]
    async fn rejects_non_https_url() {
        let mut entry = catalog::find("ormbg").unwrap().clone();
        entry.url = "http://example.invalid/model.onnx";
        let err = fetch_catalog_model(&entry).await.unwrap_err();
        assert!(matches!(err, DownloadError::InsecureUrl));
    }

    #[tokio::test]
    async fn every_catalog_url_is_https() {
        for entry in catalog::CATALOG {
            assert!(entry.url.starts_with("https://"), "{}", entry.id);
        }
    }
}
