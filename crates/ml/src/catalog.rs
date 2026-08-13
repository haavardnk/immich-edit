mod entries;
mod semantic;

use crate::model::{ModelKind, ModelSpec};

pub use entries::CATALOG;
pub use semantic::{SEMANTIC_CLASSES, SemanticClass, semantic_class};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    Recommended,
    Alternative,
    LowMemory,
}

#[derive(Debug, Clone)]
pub struct Cost {
    pub gpu_ms: u32,
    pub gpu_mb: u32,
    pub cpu_ms: u32,
    pub cpu_mb: u32,
}

#[derive(Debug, Clone)]
pub struct CatalogFile {
    pub url: &'static str,
    pub sha256: &'static str,
    pub size_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct CatalogEntry {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: ModelKind,
    pub tier: Tier,
    pub url: &'static str,
    pub sha256: &'static str,
    pub size_bytes: u64,
    pub aux: Option<CatalogFile>,
    pub license: &'static str,
    pub source: &'static str,
    pub notes: &'static str,
    pub spec: ModelSpec,
    pub cost: Cost,
}

impl CatalogEntry {
    pub fn total_bytes(&self) -> u64 {
        self.size_bytes + self.aux.as_ref().map_or(0, |a| a.size_bytes)
    }
}

pub const KINDS: &[ModelKind] = &[
    ModelKind::Subject,
    ModelKind::People,
    ModelKind::Sky,
    ModelKind::Depth,
    ModelKind::Semantic,
    ModelKind::Click,
];

pub fn find(id: &str) -> Option<&'static CatalogEntry> {
    CATALOG.iter().find(|e| e.id == id)
}

pub fn for_kind(kind: ModelKind) -> impl Iterator<Item = &'static CatalogEntry> {
    CATALOG.iter().filter(move |e| e.kind == kind)
}

pub fn default_for(kind: ModelKind) -> Option<&'static CatalogEntry> {
    for_kind(kind).find(|e| e.tier == Tier::Recommended)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique() {
        let mut ids: Vec<&str> = CATALOG.iter().map(|e| e.id).collect();
        ids.sort_unstable();
        let count = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), count);
    }

    #[test]
    fn every_kind_has_exactly_one_recommended() {
        for kind in KINDS {
            let n = for_kind(*kind)
                .filter(|e| e.tier == Tier::Recommended)
                .count();
            assert_eq!(n, 1, "{} has {n} recommended entries", kind.as_str());
        }
    }

    #[test]
    fn hashes_are_well_formed() {
        for e in CATALOG {
            assert_eq!(e.sha256.len(), 64, "{}", e.id);
            assert!(e.sha256.chars().all(|c| c.is_ascii_hexdigit()), "{}", e.id);
            assert!(e.size_bytes > 0, "{}", e.id);
            assert!(e.url.starts_with("https://"), "{}", e.id);
            if let Some(aux) = &e.aux {
                assert_eq!(aux.sha256.len(), 64, "{} aux", e.id);
                assert!(
                    aux.sha256.chars().all(|c| c.is_ascii_hexdigit()),
                    "{} aux",
                    e.id
                );
                assert!(aux.size_bytes > 0, "{} aux", e.id);
                assert!(aux.url.starts_with("https://"), "{} aux", e.id);
                assert_ne!(aux.sha256, e.sha256, "{} aux", e.id);
            }
        }
    }

    #[test]
    fn subject_default_is_ormbg() {
        assert_eq!(default_for(ModelKind::Subject).unwrap().id, "ormbg");
    }
}
