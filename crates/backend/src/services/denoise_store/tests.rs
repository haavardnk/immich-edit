use super::*;

fn key(source: u128) -> PlaneKey {
    PlaneKey {
        server_epoch: 1,
        owner: Uuid::nil(),
        source: Uuid::from_u128(source),
        model: "nind_utnet".into(),
    }
}

fn plane(key: &PlaneKey) -> DenoisePlane {
    let bits = 0x3400;
    DenoisePlane::new(key.plane_id(), 6, 4, vec![bits; 6 * 4 * 3]).unwrap()
}

async fn put(store: &DenoiseStore, key: &PlaneKey, plane: DenoisePlane) {
    let ticket = store.ticket(key.owner).await;
    assert!(
        store
            .put(key.clone(), plane, ticket)
            .await
            .unwrap()
            .is_some()
    );
}

#[test]
fn plane_ids_follow_every_key_field() {
    let base = key(1);
    let other_model = PlaneKey {
        model: "other".into(),
        ..base.clone()
    };
    let other_epoch = PlaneKey {
        server_epoch: 2,
        ..base.clone()
    };
    assert_ne!(base.plane_id(), key(2).plane_id());
    assert_ne!(base.plane_id(), other_model.plane_id());
    assert_ne!(base.plane_id(), other_epoch.plane_id());
    assert_eq!(base.plane_id(), key(1).plane_id());
    let named = Sha256::digest(format!(
        "1/{}/{}/nind_utnet/r{BAKE_REVISION}",
        base.owner, base.source
    ));
    assert_eq!(base.plane_id().to_le_bytes(), named[..8]);
    assert!(
        base.file_name()
            .ends_with(&format!("-r{BAKE_REVISION}.{EXTENSION}"))
    );
}

#[tokio::test]
async fn a_plane_survives_memory_eviction_via_disk() {
    let dir = tempfile::tempdir().unwrap();
    let store = DenoiseStore::new(dir.path(), 64).unwrap();
    let first = key(1);
    put(&store, &first, plane(&first)).await;
    for i in 2..=(MEMORY_SLOTS.get() as u128 + 2) {
        let k = key(i);
        put(&store, &k, plane(&k)).await;
    }
    assert_eq!(store.memory.lock().await.len(), MEMORY_SLOTS.get());
    assert!(store.contains(&first).await);
    assert_eq!(*store.get(&first).await.unwrap(), plane(&first));
    assert!(store.get(&key(99)).await.is_none());
}

#[tokio::test]
async fn a_plane_written_for_another_key_is_discarded_and_untracked() {
    let dir = tempfile::tempdir().unwrap();
    let store = DenoiseStore::new(dir.path(), 64).unwrap();
    let wanted = key(1);
    put(&store, &wanted, plane(&key(2))).await;
    store.memory.lock().await.clear();
    assert!(store.disk_bytes().await.0 > 0);
    assert!(store.get(&wanted).await.is_none());
    assert!(!store.contains(&wanted).await);
    assert_eq!(store.disk_bytes().await.0, 0);
}

#[tokio::test]
async fn ready_sources_lists_the_owners_planes_for_one_model() {
    let dir = tempfile::tempdir().unwrap();
    let store = DenoiseStore::new(dir.path(), 64).unwrap();
    let first = key(1);
    let second = key(2);
    let other_model = PlaneKey {
        model: "other".into(),
        ..key(3)
    };
    for k in [&first, &second, &other_model] {
        put(&store, k, plane(k)).await;
    }
    store.memory.lock().await.clear();
    let ready = store.ready_sources(1, Uuid::nil(), "nind_utnet").await;
    assert_eq!(ready, HashSet::from([first.source, second.source]));
    assert!(
        store
            .ready_sources(1, Uuid::from_u128(9), "nind_utnet")
            .await
            .is_empty()
    );
}

#[tokio::test]
async fn a_purge_drops_planes_and_puts_from_before_it() {
    let dir = tempfile::tempdir().unwrap();
    let store = DenoiseStore::new(dir.path(), 64).unwrap();
    let k = key(7);
    let other = PlaneKey {
        owner: Uuid::from_u128(9),
        ..key(8)
    };
    put(&store, &k, plane(&k)).await;
    put(&store, &other, plane(&other)).await;
    let stale = store.ticket(k.owner).await;
    let unaffected = store.ticket(other.owner).await;
    store.purge_owner(k.owner).await;
    assert!(!store.contains(&k).await);
    assert!(store.contains(&other).await);
    assert!(
        store
            .put(k.clone(), plane(&k), stale)
            .await
            .unwrap()
            .is_none()
    );
    assert!(!store.contains(&k).await);
    assert!(
        store
            .put(other.clone(), plane(&other), unaffected)
            .await
            .unwrap()
            .is_some()
    );

    let before_all = store.ticket(k.owner).await;
    store.purge_all().await;
    assert!(!store.contains(&other).await);
    assert!(
        store
            .put(k.clone(), plane(&k), before_all)
            .await
            .unwrap()
            .is_none()
    );
    assert!(!store.contains(&k).await);
    assert_eq!(store.disk_bytes().await.0, 0);
}

#[tokio::test]
async fn a_read_that_races_a_purge_is_not_remembered() {
    let dir = tempfile::tempdir().unwrap();
    let store = DenoiseStore::new(dir.path(), 64).unwrap();
    let k = key(10);
    let path = store.path(&k).unwrap();
    let stale = store.ticket(k.owner).await;
    store.purge_owner(k.owner).await;
    assert!(store.remember(&k, &path, plane(&k), stale).await.is_none());
    assert!(!store.contains(&k).await);
    let current = store.ticket(k.owner).await;
    assert!(
        store
            .remember(&k, &path, plane(&k), current)
            .await
            .is_some()
    );
}

#[tokio::test]
async fn a_plane_larger_than_the_budget_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let store = DenoiseStore::new(dir.path(), 1).unwrap();
    let k = key(11);
    let (w, h) = (800, 800);
    let mut seed = 0x9e37_79b9_u32;
    let noise = (0..w * h * 3)
        .map(|_| {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            0x3000 + (seed >> 22) as u16
        })
        .collect();
    let large = DenoisePlane::new(k.plane_id(), w, h, noise).unwrap();
    let ticket = store.ticket(k.owner).await;
    let error = store.put(k.clone(), large, ticket).await.unwrap_err();
    assert!(error.to_string().contains("DENOISE_CACHE_MB"), "{error}");
    assert!(!store.contains(&k).await);
    assert_eq!(store.disk_bytes().await.0, 0);
}
