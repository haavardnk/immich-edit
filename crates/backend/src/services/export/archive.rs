use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

pub fn zip_job_dir(state: &AppState, server_epoch: i64, owner: Uuid, job_id: Uuid) -> PathBuf {
    state
        .config
        .cache_dir
        .join("exports")
        .join(server_epoch.to_string())
        .join(owner.to_string())
        .join(job_id.to_string())
}

pub fn zip_archive_path(state: &AppState, server_epoch: i64, owner: Uuid, job_id: Uuid) -> PathBuf {
    state
        .config
        .cache_dir
        .join("exports")
        .join(server_epoch.to_string())
        .join(owner.to_string())
        .join(format!("{job_id}.zip"))
}

pub async fn cleanup_zip_job(state: &AppState, server_epoch: i64, owner: Uuid, job_id: Uuid) {
    let dir = zip_job_dir(state, server_epoch, owner, job_id);
    if let Err(e) = tokio::fs::remove_dir_all(&dir).await
        && e.kind() != std::io::ErrorKind::NotFound
    {
        tracing::warn!(error = %e, "remove export dir");
    }
    let archive = zip_archive_path(state, server_epoch, owner, job_id);
    if let Err(e) = tokio::fs::remove_file(&archive).await
        && e.kind() != std::io::ErrorKind::NotFound
    {
        tracing::warn!(error = %e, "remove export archive");
    }
}

pub(super) fn sanitize_filename(name: &str) -> String {
    let stem = match name.rsplit_once('.') {
        Some((s, _)) => s,
        None => name,
    };
    let cleaned: String = stem
        .chars()
        .map(|c| {
            if c.is_control() || matches!(c, '/' | '\\' | '\0' | ':') {
                '_'
            } else {
                c
            }
        })
        .collect();
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        "export".into()
    } else {
        trimmed.to_string()
    }
}

pub(super) async fn write_unique(
    dir: &Path,
    stem: &str,
    suffix: &str,
    extension: &str,
    bytes: &[u8],
) -> std::io::Result<String> {
    let mut n: u32 = 1;
    loop {
        let filename = if n == 1 {
            format!("{stem}{suffix}.{extension}")
        } else {
            format!("{stem}{suffix}_{n}.{extension}")
        };
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dir.join(&filename))
            .await
        {
            Ok(mut file) => {
                use tokio::io::AsyncWriteExt;
                file.write_all(bytes).await?;
                file.flush().await?;
                return Ok(filename);
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                n += 1;
            }
            Err(e) => return Err(e),
        }
    }
}

pub async fn build_zip_archive(
    state: &AppState,
    server_epoch: i64,
    owner: Uuid,
    job_id: Uuid,
) -> Result<PathBuf, AppError> {
    let archive = zip_archive_path(state, server_epoch, owner, job_id);
    if tokio::fs::try_exists(&archive).await.unwrap_or(false) {
        return Ok(archive);
    }
    let dir = zip_job_dir(state, server_epoch, owner, job_id);
    let archive_for_task = archive.clone();
    tokio::task::spawn_blocking(move || zip_dir_blocking(&dir, &archive_for_task))
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "zip task join");
            AppError::Internal
        })?
        .map_err(|e| {
            tracing::error!(error = %e, "build zip");
            AppError::Internal
        })?;
    Ok(archive)
}

pub async fn purge_owner_exports(state: &AppState, owner: Uuid) {
    let root = state.config.cache_dir.join("exports");
    let Ok(mut epochs) = tokio::fs::read_dir(root).await else {
        return;
    };
    while let Ok(Some(epoch)) = epochs.next_entry().await {
        let path = epoch.path().join(owner.to_string());
        if let Err(error) = tokio::fs::remove_dir_all(path).await
            && error.kind() != std::io::ErrorKind::NotFound
        {
            tracing::warn!(error = %error, %owner, "remove owner exports");
        }
    }
}

pub async fn purge_all_exports(state: &AppState) {
    let root = state.config.cache_dir.join("exports");
    if let Err(error) = tokio::fs::remove_dir_all(&root).await
        && error.kind() != std::io::ErrorKind::NotFound
    {
        tracing::warn!(error = %error, "remove all exports");
    }
    let _ = tokio::fs::create_dir_all(root).await;
}

fn zip_dir_blocking(dir: &Path, archive: &Path) -> std::io::Result<()> {
    let tmp = archive.with_extension("zip.part");
    let file = std::fs::File::create(&tmp)?;
    let mut writer = zip::ZipWriter::new(file);
    let options: zip::write::SimpleFileOptions =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        writer.start_file(name, options)?;
        let mut src = std::fs::File::open(entry.path())?;
        std::io::copy(&mut src, &mut writer)?;
    }
    let mut out = writer.finish()?;
    out.flush()?;
    std::fs::rename(&tmp, archive)?;
    Ok(())
}
