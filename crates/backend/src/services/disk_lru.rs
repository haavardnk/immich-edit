use std::path::{Path, PathBuf};
use std::time::SystemTime;

use lru::LruCache;
use tokio::fs;
use tokio::sync::Mutex;

use crate::services::cleanup::warn_unless_missing;

const MB: u64 = 1024 * 1024;

struct DiskState {
    lru: LruCache<PathBuf, u64>,
    total_bytes: u64,
    cap_bytes: u64,
}

pub struct DiskLru {
    state: Mutex<DiskState>,
}

fn collect_files(
    dir: &Path,
    extension: &str,
    out: &mut Vec<(PathBuf, u64, SystemTime)>,
) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let meta = entry.metadata()?;
        if meta.is_dir() {
            collect_files(&path, extension, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some(extension) {
            out.push((path, meta.len(), meta.modified()?));
        }
    }
    Ok(())
}

impl DiskLru {
    pub fn open(dir: &Path, extension: &str, cap_mb: u64) -> std::io::Result<Self> {
        std::fs::create_dir_all(dir)?;
        let mut entries: Vec<(PathBuf, u64, SystemTime)> = Vec::new();
        collect_files(dir, extension, &mut entries)?;
        entries.sort_by_key(|(_, _, modified)| *modified);

        let mut lru = LruCache::unbounded();
        let mut total_bytes = 0u64;
        for (path, size, _) in entries {
            total_bytes = total_bytes.saturating_add(size);
            lru.put(path, size);
        }
        let mut state = DiskState {
            lru,
            total_bytes,
            cap_bytes: cap_mb.saturating_mul(MB),
        };
        while state.total_bytes > state.cap_bytes {
            let Some((path, size)) = state.lru.pop_lru() else {
                break;
            };
            state.total_bytes = state.total_bytes.saturating_sub(size);
            warn_unless_missing(std::fs::remove_file(&path), &path);
        }
        Ok(Self {
            state: Mutex::new(state),
        })
    }

    pub async fn bytes(&self) -> (u64, u64) {
        let st = self.state.lock().await;
        (st.total_bytes, st.cap_bytes)
    }

    pub async fn record(&self, path: PathBuf, size: u64) {
        {
            let mut st = self.state.lock().await;
            if let Some(previous) = st.lru.pop(&path) {
                st.total_bytes = st.total_bytes.saturating_sub(previous);
            }
            st.total_bytes = st.total_bytes.saturating_add(size);
            st.lru.put(path, size);
        }
        self.evict_to_cap().await;
    }

    pub async fn touch(&self, path: &Path) {
        self.state.lock().await.lru.get(path);
    }

    pub async fn forget_under(&self, dir: &Path) {
        let mut st = self.state.lock().await;
        let stale: Vec<PathBuf> = st
            .lru
            .iter()
            .map(|(path, _)| path.clone())
            .filter(|path| path.starts_with(dir))
            .collect();
        for path in stale {
            if let Some(size) = st.lru.pop(&path) {
                st.total_bytes = st.total_bytes.saturating_sub(size);
            }
        }
    }

    async fn evict_to_cap(&self) {
        loop {
            let victim = {
                let mut st = self.state.lock().await;
                if st.total_bytes <= st.cap_bytes {
                    return;
                }
                match st.lru.pop_lru() {
                    Some((path, size)) => {
                        st.total_bytes = st.total_bytes.saturating_sub(size);
                        path
                    }
                    None => return,
                }
            };
            warn_unless_missing(fs::remove_file(&victim).await, &victim);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn opening_trims_an_oversized_directory_oldest_first() {
        let dir = tempfile::tempdir().unwrap();
        let old = dir.path().join("a/old.bin");
        let new = dir.path().join("new.bin");
        std::fs::create_dir_all(old.parent().unwrap()).unwrap();
        std::fs::write(&old, vec![0u8; 700 * 1024]).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
        std::fs::write(&new, vec![0u8; 700 * 1024]).unwrap();
        std::fs::write(dir.path().join("ignored.txt"), b"x").unwrap();
        let lru = DiskLru::open(dir.path(), "bin", 1).unwrap();
        assert!(!old.exists());
        assert!(new.exists());
        assert_eq!(lru.bytes().await, (700 * 1024, MB));
    }

    #[tokio::test]
    async fn recording_past_the_cap_evicts_the_least_recent_file() {
        let dir = tempfile::tempdir().unwrap();
        let lru = DiskLru::open(dir.path(), "bin", 1).unwrap();
        let paths: Vec<PathBuf> = (0..3)
            .map(|i| dir.path().join(format!("{i}.bin")))
            .collect();
        for path in &paths[..2] {
            std::fs::write(path, vec![0u8; 400 * 1024]).unwrap();
            lru.record(path.clone(), 400 * 1024).await;
        }
        lru.touch(&paths[0]).await;
        std::fs::write(&paths[2], vec![0u8; 400 * 1024]).unwrap();
        lru.record(paths[2].clone(), 400 * 1024).await;
        assert!(paths[0].exists());
        assert!(!paths[1].exists());
        assert!(paths[2].exists());
        lru.forget_under(dir.path()).await;
        assert_eq!(lru.bytes().await.0, 0);
    }

    #[tokio::test]
    async fn entry_count_never_drops_tracked_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let lru = DiskLru::open(dir.path(), "bin", 1).unwrap();
        let count = 9000u64;
        for i in 0..count {
            lru.record(dir.path().join(format!("{i}.bin")), 1).await;
        }
        assert_eq!(lru.bytes().await.0, count);
        lru.forget_under(dir.path()).await;
        assert_eq!(lru.bytes().await.0, 0);
    }
}
