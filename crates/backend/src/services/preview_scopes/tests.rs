use raw_pipeline::histogram::Histogram;

use super::*;
use crate::services::preview_meta::PreviewMeta;

fn grids() -> ScopeGrids {
    ScopeGrids::from_rgb_u8(&[10, 20, 30, 40, 50, 60], 2, 1)
}

fn meta(owner: Uuid, asset_id: AssetKey) -> PreviewMeta {
    PreviewMeta {
        owner,
        asset_id,
        width: 2,
        height: 1,
        source_w: 2,
        source_h: 1,
        renderer: "cpu".into(),
        is_raw: true,
        histogram: Histogram::from_rgb_u8(&[0, 0, 0], 1, 1),
        linear_histogram: None,
        has_scopes: true,
    }
}

#[tokio::test]
async fn encode_owned_serves_only_the_owner_of_that_asset() {
    let metas = PreviewMetaStore::new();
    let scopes = PreviewScopeStore::new();
    let owner = Uuid::new_v4();
    let asset = AssetKey::master(Uuid::new_v4());
    let with_grids = Uuid::new_v4();
    let without_grids = Uuid::new_v4();
    metas.put(with_grids, meta(owner, asset)).await;
    metas.put(without_grids, meta(owner, asset)).await;
    scopes.put(with_grids, grids()).await;
    let other_asset = AssetKey::master(Uuid::new_v4());
    let cases = [
        (owner, asset, with_grids, true),
        (Uuid::new_v4(), asset, with_grids, false),
        (owner, other_asset, with_grids, false),
        (owner, asset, without_grids, false),
        (owner, asset, Uuid::new_v4(), false),
    ];
    for (who, asset_id, meta_id, served) in cases {
        let encoded = scopes
            .encode_owned(&metas, who, asset_id, meta_id, ScopeKind::Parade)
            .await;
        assert_eq!(encoded.is_some(), served, "{who} {asset_id:?} {meta_id}");
    }
    let encoded = scopes
        .encode_owned(&metas, owner, asset, with_grids, ScopeKind::Parade)
        .await;
    assert_eq!(encoded, Some(encode(&grids(), ScopeKind::Parade)));
}

#[tokio::test]
async fn lru_caps_and_evicts() {
    let store = PreviewScopeStore::with_capacity(2);
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    let c = Uuid::new_v4();
    store.put(a, grids()).await;
    store.put(b, grids()).await;
    store.put(c, grids()).await;
    if store.len().await != 2 {
        panic!("expected 2 entries");
    }
    if store.get(a).await.is_some() {
        panic!("oldest entry should have been evicted");
    }
    if store.get(c).await.is_none() {
        panic!("newest entry should be present");
    }
}

#[tokio::test]
async fn clear_drops_everything() {
    let store = PreviewScopeStore::new();
    let id = Uuid::new_v4();
    store.put(id, grids()).await;
    store.clear().await;
    if store.get(id).await.is_some() {
        panic!("clear should empty the store");
    }
}

#[test]
fn encode_prefixes_a_readable_header() {
    let grids = grids();
    for (kind, tag) in [
        (ScopeKind::Waveform, 0u8),
        (ScopeKind::Parade, 1),
        (ScopeKind::Vectorscope, 2),
    ] {
        let grid = kind.select(&grids);
        let bytes = encode(&grids, kind);
        assert_eq!(&bytes[..4], b"SCOP");
        assert_eq!(bytes[4], VERSION);
        assert_eq!(bytes[5], tag);
        assert_eq!(bytes[6], grid.channels);
        assert_eq!(u16::from_le_bytes([bytes[8], bytes[9]]), grid.width);
        assert_eq!(u16::from_le_bytes([bytes[10], bytes[11]]), grid.height);
        assert_eq!(
            u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
            grid.max_count
        );
        assert_eq!(bytes.len(), HEADER_LEN + grid.data.len());
        assert_eq!(&bytes[HEADER_LEN..], &grid.data[..]);
    }
}
