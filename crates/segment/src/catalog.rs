use crate::model::{ModelKind, ModelSpec};

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

pub const CATALOG: &[CatalogEntry] = &[
    CatalogEntry {
        id: "ormbg",
        name: "Open Remove Background",
        kind: ModelKind::Subject,
        tier: Tier::Recommended,
        url: "https://huggingface.co/schirrmacher/ormbg/resolve/main/ormbg.onnx",
        sha256: "89b47dd4fa46a76e91b06affeb5ec7881894a27792e41f7dfaf69987653f31d3",
        size_bytes: 176_182_050,
        aux: None,
        license: "Apache-2.0",
        source: "schirrmacher/ormbg",
        notes: "Soft per-strand alpha; keeps hair and small jewellery.",
        spec: ModelSpec::ORMBG,
        cost: Cost {
            gpu_ms: 742,
            gpu_mb: 431,
            cpu_ms: 6823,
            cpu_mb: 768,
        },
    },
    CatalogEntry {
        id: "u2netp",
        name: "U2-Net Lite",
        kind: ModelKind::Subject,
        tier: Tier::LowMemory,
        url: "https://github.com/danielgatis/rembg/releases/download/v0.0.0/u2netp.onnx",
        sha256: "309c8469258dda742793dce0ebea8e6dd393174f89934733ecc8b14c76f4ddd8",
        size_bytes: 4_574_861,
        aux: None,
        license: "Apache-2.0",
        source: "xuebinqin/U-2-Net",
        notes: "320px only; blobby at high resolution. For constrained hosts.",
        spec: ModelSpec::U2NET,
        cost: Cost {
            gpu_ms: 116,
            gpu_mb: 68,
            cpu_ms: 774,
            cpu_mb: 233,
        },
    },
    CatalogEntry {
        id: "modnet",
        name: "MODNet",
        kind: ModelKind::People,
        tier: Tier::Recommended,
        url: "https://huggingface.co/onnx-community/modnet-webnn/resolve/main/onnx/model.onnx",
        sha256: "07c308cf0fc7e6e8b2065a12ed7fc07e1de8febb7dc7839d7b7f15dd66584df9",
        size_bytes: 25_888_640,
        aux: None,
        license: "Apache-2.0",
        source: "ZHKKKe/MODNet",
        notes: "Tuned for people. Softer edges around hair than the subject models, and the fastest model here.",
        spec: ModelSpec::MODNET,
        cost: Cost {
            gpu_ms: 140,
            gpu_mb: 122,
            cpu_ms: 195,
            cpu_mb: 196,
        },
    },
    CatalogEntry {
        id: "skyseg",
        name: "Sky Segmentation",
        kind: ModelKind::Sky,
        tier: Tier::Recommended,
        url: "https://huggingface.co/JianyuanWang/skyseg/resolve/main/skyseg.onnx",
        sha256: "ab9c34c64c3d821220a2886a4a06da4642ffa14d5b30e8d5339056a089aa1d39",
        size_bytes: 175_997_079,
        aux: None,
        license: "MIT",
        source: "xiongzhu666/Sky-Segmentation-and-Post-processing",
        notes: "Picks out sky for skies, horizons and reflections.",
        spec: ModelSpec::SKYSEG,
        cost: Cost {
            gpu_ms: 260,
            gpu_mb: 431,
            cpu_ms: 4207,
            cpu_mb: 406,
        },
    },
    CatalogEntry {
        id: "depth_anything_v2_vits",
        name: "Depth Anything V2 Small",
        kind: ModelKind::Depth,
        tier: Tier::LowMemory,
        url: "https://huggingface.co/CyberTimon/RapidRAW-Models/resolve/main/depth_anything_v2_vits.onnx",
        sha256: "d2b11a11c1d4a12b47608fa65a17ee9a4c605b55ee1730c8e3b526304f2562be",
        size_bytes: 99_373_606,
        aux: None,
        license: "Apache-2.0",
        source: "DepthAnything/Depth-Anything-V2",
        notes: "A quarter of the download and twice the speed of Base. Depth edges are softer.",
        spec: ModelSpec::DEPTH_ANYTHING,
        cost: Cost {
            gpu_ms: 339,
            gpu_mb: 284,
            cpu_ms: 2913,
            cpu_mb: 350,
        },
    },
    CatalogEntry {
        id: "depth_anything_v2_vitb",
        name: "Depth Anything V2 Base",
        kind: ModelKind::Depth,
        tier: Tier::Recommended,
        url: "https://huggingface.co/onnx-community/depth-anything-v2-base/resolve/main/onnx/model.onnx",
        sha256: "dab40f67d5ead5e0ed52acfae41a717a708b02bc57f6cb761c5de828f26a7289",
        size_bytes: 388_917_010,
        aux: None,
        license: "CC-BY-NC-4.0",
        source: "DepthAnything/Depth-Anything-V2",
        notes: "Estimates distance from camera, so you can mask by depth: near subject, mid ground, or far background.",
        spec: ModelSpec::DEPTH_ANYTHING,
        cost: Cost {
            gpu_ms: 873,
            gpu_mb: 761,
            cpu_ms: 1810,
            cpu_mb: 726,
        },
    },
    CatalogEntry {
        id: "mobilesam",
        name: "MobileSAM",
        kind: ModelKind::Click,
        tier: Tier::Recommended,
        url: "https://huggingface.co/CyberTimon/RapidRAW-Models/resolve/main/vit_t_encoder.onnx",
        sha256: "8b8168033ea6687bb55ba242222b67a301ac9da30fd5cbfd04dcebbb180ec2a8",
        size_bytes: 28_106_845,
        aux: Some(CatalogFile {
            url: "https://huggingface.co/CyberTimon/RapidRAW-Models/resolve/main/vit_t_decoder.onnx",
            sha256: "1b216fb3b8ceeee00a65f89670c01e4c0d823fcacec39dd9accc233f85341dc4",
            size_bytes: 16_501_324,
        }),
        license: "Apache-2.0",
        source: "ChaoningZhang/MobileSAM",
        notes: "Click to add or remove parts of a mask. Encodes the photo once, then each click is fast.",
        spec: ModelSpec::MOBILE_SAM,
        cost: Cost {
            gpu_ms: 1131,
            gpu_mb: 512,
            cpu_ms: 4200,
            cpu_mb: 600,
        },
    },
];

pub const KINDS: &[ModelKind] = &[
    ModelKind::Subject,
    ModelKind::People,
    ModelKind::Sky,
    ModelKind::Depth,
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
