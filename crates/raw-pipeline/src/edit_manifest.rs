use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::edits::Edits;
use crate::ops::{OpRegistry, default_registry};

pub const EDIT_MANIFEST_VERSION: u32 = 4;

type Migration = fn(&mut BTreeMap<String, Value>);

const MIGRATIONS: &[(u32, Migration)] = &[(4, migrate_v4_explicit_lens_profile)];

fn migrate_v4_explicit_lens_profile(ops: &mut BTreeMap<String, Value>) {
    let entry = ops
        .entry(crate::ops::lens_profile::LENS_PROFILE_OP_ID.to_string())
        .or_insert_with(|| Value::Object(serde_json::Map::new()));
    let Some(map) = entry.as_object_mut() else {
        return;
    };
    map.entry("profile_enabled").or_insert(Value::Bool(false));
}

fn migrate_in_place(from: u32, ops: &mut BTreeMap<String, Value>) {
    for (target, mig) in MIGRATIONS {
        if from < *target {
            mig(ops);
        }
    }
}

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
        for (key, value) in &edits.unknown_ops {
            ops.entry(key.clone()).or_insert_with(|| value.clone());
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
        let mut ops = self.ops.clone();
        if self.schema_version < EDIT_MANIFEST_VERSION {
            migrate_in_place(self.schema_version, &mut ops);
        }
        let mut edits = Edits::default();
        let mut known: Vec<&str> = Vec::with_capacity(registry.ops().len());
        for op in registry.ops() {
            known.push(op.id());
            if let Some(value) = ops.get(op.id()) {
                op.from_doc(value, &mut edits);
            }
        }
        for (key, value) in ops {
            if !known.iter().any(|id| *id == key) {
                edits.unknown_ops.insert(key, value);
            }
        }
        edits
    }
}

#[cfg(test)]
mod tests;
