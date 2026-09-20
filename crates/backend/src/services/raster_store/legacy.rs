use std::path::{Path, PathBuf};

fn relative_files(root: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(current) = pending.pop() {
        for entry in std::fs::read_dir(&current)? {
            let entry = entry?;
            let path = entry.path();
            if entry.file_type()?.is_dir() {
                pending.push(path);
                continue;
            }
            let Ok(relative) = path.strip_prefix(root) else {
                continue;
            };
            files.push(relative.to_path_buf());
        }
    }
    Ok(files)
}

fn remove_empty_dirs(root: &Path) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        if entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
            remove_empty_dirs(&entry.path());
        }
    }
    let _ = std::fs::remove_dir(root);
}

pub fn migrate_legacy_layout(data_dir: &Path) -> std::io::Result<usize> {
    let legacy = data_dir.join("cache").join("rasters");
    if !legacy.is_dir() {
        return Ok(0);
    }
    let dir = data_dir.join("rasters");
    if !dir.exists() {
        let moved = relative_files(&legacy)?.len();
        if let Some(parent) = dir.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::rename(&legacy, &dir)?;
        return Ok(moved);
    }
    let mut moved = 0;
    for relative in relative_files(&legacy)? {
        let source = legacy.join(&relative);
        let target = dir.join(&relative);
        if target.exists() {
            continue;
        }
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::rename(&source, &target)?;
        moved += 1;
    }
    remove_empty_dirs(&legacy);
    Ok(moved)
}
