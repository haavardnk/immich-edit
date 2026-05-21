use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::edits::Edits;
use crate::ops::{OpRegistry, default_registry};

pub const EDIT_MANIFEST_VERSION: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EditManifest {
    pub schema_version: u32,
    pub ops: BTreeMap<String, Value>,
}

impl Default for EditManifest {
    fn default() -> Self {
        Self {
            schema_version: EDIT_MANIFEST_VERSION,
            ops: BTreeMap::new(),
        }
    }
}

impl EditManifest {
    pub fn from_edits(edits: &Edits) -> Self {
        Self::from_edits_with(edits, &default_registry())
    }

    pub fn from_edits_with(edits: &Edits, registry: &OpRegistry) -> Self {
        let mut ops = BTreeMap::new();
        for op in registry.ops() {
            if let Some(value) = op.to_doc(edits) {
                ops.insert(op.id().to_string(), value);
            }
        }
        Self {
            schema_version: EDIT_MANIFEST_VERSION,
            ops,
        }
    }

    pub fn to_edits(&self) -> Edits {
        self.to_edits_with(&default_registry())
    }

    pub fn to_edits_with(&self, registry: &OpRegistry) -> Edits {
        let mut edits = Edits::default();
        for op in registry.ops() {
            if let Some(value) = self.ops.get(op.id()) {
                op.from_doc(value, &mut edits);
            }
        }
        edits
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edits::{BasicEdits, CropRect, GeometryEdits, ToneEdits};

    #[test]
    fn empty_edits_yields_empty_doc() {
        let doc = EditManifest::from_edits(&Edits::default());
        if !doc.ops.is_empty() {
            panic!("expected no ops, got {:?}", doc.ops);
        }
        if doc.schema_version != EDIT_MANIFEST_VERSION {
            panic!("wrong version");
        }
    }

    #[test]
    fn roundtrip_preserves_fields() {
        let original = Edits {
            basic: BasicEdits {
                exposure_ev: 1.5,
                contrast: 25.0,
                saturation: 12.5,
                wb_temp: 8.0,
                wb_tint: -4.0,
            },
            tone: ToneEdits {
                highlights: -10.0,
                shadows: 30.0,
            },
            geometry: GeometryEdits {
                rotate: 90,
                flip_h: true,
                flip_v: false,
                crop: Some(CropRect {
                    x: 0.1,
                    y: 0.2,
                    width: 0.5,
                    height: 0.6,
                }),
            },
        };
        let doc = EditManifest::from_edits(&original);
        let back = doc.to_edits();
        if back != original {
            panic!("roundtrip mismatch: {back:?} != {original:?}");
        }
    }

    #[test]
    fn sparse_doc_only_includes_active_ops() {
        let edits = Edits {
            basic: BasicEdits {
                exposure_ev: 0.5,
                ..Default::default()
            },
            ..Default::default()
        };
        let doc = EditManifest::from_edits(&edits);
        if doc.ops.len() != 1 || !doc.ops.contains_key("exposure") {
            panic!(
                "expected only exposure key, got {:?}",
                doc.ops.keys().collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn from_doc_ignores_unknown_keys() {
        let mut ops = BTreeMap::new();
        ops.insert("exposure".into(), serde_json::json!({ "ev": 2.0 }));
        ops.insert("nonexistent".into(), serde_json::json!({ "foo": 1 }));
        let doc = EditManifest {
            schema_version: EDIT_MANIFEST_VERSION,
            ops,
        };
        let edits = doc.to_edits();
        if edits.basic.exposure_ev != 2.0 {
            panic!("exposure not parsed");
        }
    }
}
