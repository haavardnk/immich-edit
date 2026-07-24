use std::path::Path;

use sha2::{Digest, Sha256};
use tokio::fs;
use uuid::Uuid;

pub fn content_hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

pub async fn write_blob_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if fs::try_exists(path).await? {
        return Ok(());
    }
    let Some(dir) = path.parent() else {
        return fs::write(path, bytes).await;
    };
    let tmp = dir.join(format!(".tmp-{}", Uuid::new_v4()));
    fs::write(&tmp, bytes).await?;
    if let Err(e) = fs::rename(&tmp, path).await {
        let _ = fs::remove_file(&tmp).await;
        return Err(e);
    }
    Ok(())
}

pub fn is_unique_violation(err: &sqlx::Error) -> bool {
    err.as_database_error()
        .is_some_and(|e| e.is_unique_violation())
}
