use uuid::Uuid;

use crate::asset_key::AssetKey;
use crate::immich::dto::{AssetDetail, AssetSummary};
use crate::services::edits_store::{CopyRecord, EditsStore, EditsStoreError};

pub trait CopyExpandable: Clone {
    fn key(&self) -> AssetKey;
    fn apply_copy(&mut self, copy: &CopyRecord);
}

impl CopyExpandable for AssetSummary {
    fn key(&self) -> AssetKey {
        self.id
    }

    fn apply_copy(&mut self, copy: &CopyRecord) {
        self.id = copy.id;
        self.copy_of = Some(copy.source_asset_id);
        self.copy_label = copy.name.clone();
    }
}

impl CopyExpandable for AssetDetail {
    fn key(&self) -> AssetKey {
        self.id
    }

    fn apply_copy(&mut self, copy: &CopyRecord) {
        self.id = copy.id;
        self.copy_of = Some(copy.source_asset_id);
        self.copy_label = copy.name.clone();
    }
}

pub async fn expand_assets<T: CopyExpandable>(
    edits: &EditsStore,
    owner: Uuid,
    items: Vec<T>,
) -> Result<Vec<T>, EditsStoreError> {
    let sources: Vec<Uuid> = items
        .iter()
        .map(|item| item.key())
        .filter(|key| !key.is_copy())
        .map(|key| key.source())
        .collect();
    let copies = edits.expand_copies(owner, &sources).await?;
    if copies.is_empty() {
        return Ok(items);
    }
    let mut out = Vec::with_capacity(items.len() + copies.values().map(Vec::len).sum::<usize>());
    for item in items {
        let key = item.key();
        let siblings = match key.is_copy() {
            true => None,
            false => copies.get(&key.source()),
        };
        let Some(siblings) = siblings else {
            out.push(item);
            continue;
        };
        let expanded: Vec<T> = siblings
            .iter()
            .map(|copy| {
                let mut clone = item.clone();
                clone.apply_copy(copy);
                clone
            })
            .collect();
        out.push(item);
        out.extend(expanded);
    }
    Ok(out)
}
