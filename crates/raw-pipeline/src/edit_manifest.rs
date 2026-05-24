use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::edits::Edits;
use crate::ops::{OpRegistry, default_registry};

pub const EDIT_MANIFEST_VERSION: u32 = 3;

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
    use crate::edits::{
        BasicEdits, ColorEdits, CurvePoints, DetailEdits, GeometryEdits, HslBand, HslEdits,
        ToneEdits,
    };

    #[test]
    fn empty_edits_yields_empty_doc() {
        let manifest = EditManifest::from_edits(&Edits::default());
        if !manifest.ops.is_empty() {
            panic!("expected no ops, got {:?}", manifest.ops);
        }
        if manifest.schema_version != EDIT_MANIFEST_VERSION {
            panic!("wrong version");
        }
    }

    #[test]
    fn roundtrip_preserves_fields() {
        let mut bands = [HslBand::default(); 8];
        bands[0] = HslBand {
            hue: 10.0,
            sat: -20.0,
            lum: 5.0,
        };
        bands[4] = HslBand {
            hue: -8.0,
            sat: 15.0,
            lum: -3.0,
        };
        let original = Edits {
            basic: BasicEdits {
                exposure_ev: 1.5,
                contrast: 25.0,
                saturation: 12.5,
                vibrance: 18.0,
                wb_temp: 8.0,
                wb_tint: -4.0,
                texture: 33.0,
                clarity: 22.0,
                dehaze: -15.0,
                curves: CurvePoints::default(),
            },
            tone: ToneEdits {
                highlights: -10.0,
                shadows: 30.0,
                blacks: 12.0,
                whites: -8.0,
            },
            color: ColorEdits {
                hsl: HslEdits { bands },
                color_grade: Default::default(),
            },
            detail: DetailEdits {
                sharpen_amount: 60.0,
                sharpen_radius: 1.2,
                sharpen_detail: 30.0,
                sharpen_masking: 15.0,
                luma_nr_amount: 25.0,
                luma_nr_detail: 45.0,
                luma_nr_contrast: 10.0,
                color_nr_amount: 40.0,
                color_nr_detail: 55.0,
                color_nr_smoothness: 60.0,
            },
            geometry: GeometryEdits {
                rotate: 90,
                rotate_angle: 0.0,
                flip_h: true,
                flip_v: false,
                crop: None,
                aspect: Default::default(),
            },
        };
        let manifest = EditManifest::from_edits(&original);
        let back = manifest.to_edits();
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
        let manifest = EditManifest::from_edits(&edits);
        if manifest.ops.len() != 1 || !manifest.ops.contains_key("exposure") {
            panic!(
                "expected only exposure key, got {:?}",
                manifest.ops.keys().collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn from_doc_ignores_unknown_keys() {
        let mut ops = BTreeMap::new();
        ops.insert("exposure".into(), serde_json::json!({ "ev": 2.0 }));
        ops.insert("nonexistent".into(), serde_json::json!({ "foo": 1 }));
        let manifest = EditManifest {
            schema_version: EDIT_MANIFEST_VERSION,
            ops,
        };
        let edits = manifest.to_edits();
        if edits.basic.exposure_ev != 2.0 {
            panic!("exposure not parsed");
        }
    }
}
