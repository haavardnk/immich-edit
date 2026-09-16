use super::*;

fn grids() -> ScopeGrids {
    ScopeGrids::from_rgb_u8(&[10, 20, 30, 40, 50, 60], 2, 1)
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
