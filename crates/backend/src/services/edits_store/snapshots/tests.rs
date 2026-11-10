use super::super::tests::{O, key, manifest_with_exposure, store};
use super::*;

async fn corrupt(s: &EditsStore, id: Uuid) {
    sqlx::query("UPDATE edit_snapshots SET edits_json = 'not json' WHERE id = ?1")
        .bind(id.to_string())
        .execute(&s.pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn list_skips_unreadable_snapshots() {
    let s = store().await;
    let id = key();
    let kept = s
        .create_snapshot(O, id, "kept", &manifest_with_exposure(0.5))
        .await
        .unwrap();
    let broken = s
        .create_snapshot(O, id, "broken", &manifest_with_exposure(1.0))
        .await
        .unwrap();
    corrupt(&s, broken.id).await;

    let listed: Vec<Uuid> = s
        .list_snapshots(O, id)
        .await
        .unwrap()
        .iter()
        .map(|snapshot| snapshot.id)
        .collect();
    if listed != vec![kept.id] {
        panic!("expected only the readable snapshot: {listed:?}");
    }
}

#[tokio::test]
async fn get_of_an_unreadable_snapshot_fails() {
    let s = store().await;
    let id = key();
    let broken = s
        .create_snapshot(O, id, "broken", &manifest_with_exposure(1.0))
        .await
        .unwrap();
    corrupt(&s, broken.id).await;

    let result = s.get_snapshot(O, id, broken.id).await;
    if !matches!(result, Err(EditsStoreError::Parse(_))) {
        panic!("expected a parse error: {result:?}");
    }
}

#[tokio::test]
async fn hash_follows_the_stored_edits() {
    let s = store().await;
    let id = key();
    let snapshot = s
        .create_snapshot(O, id, "look", &manifest_with_exposure(0.5))
        .await
        .unwrap();
    let rewritten = manifest_with_exposure(1.5).to_edits().clamped();
    sqlx::query("UPDATE edit_snapshots SET edits_json = ?2 WHERE id = ?1")
        .bind(snapshot.id.to_string())
        .bind(serde_json::to_string(&rewritten).unwrap())
        .execute(&s.pool)
        .await
        .unwrap();

    let read = s.get_snapshot(O, id, snapshot.id).await.unwrap().unwrap();
    if read.manifest_hash != rewritten.stable_hash() || read.manifest_hash == snapshot.manifest_hash
    {
        panic!("hash did not follow the stored edits: {read:?}");
    }
}

#[tokio::test]
async fn a_manifest_update_keeps_the_snapshot_identity() {
    let s = store().await;
    let id = key();
    let created = s
        .create_snapshot(O, id, "look", &manifest_with_exposure(0.5))
        .await
        .unwrap();

    let updated = s
        .update_snapshot(O, id, created.id, None, Some(&manifest_with_exposure(1.5)))
        .await
        .unwrap()
        .unwrap();

    assert_eq!(updated.id, created.id);
    assert_eq!(updated.name, "look");
    assert_eq!(updated.created_at, created.created_at);
    assert_ne!(updated.manifest_hash, created.manifest_hash);
    assert_eq!(updated.edits.basic.exposure_ev, 1.5);
    let read = s.get_snapshot(O, id, created.id).await.unwrap().unwrap();
    assert_eq!(read.manifest_hash, updated.manifest_hash);
}

#[tokio::test]
async fn an_update_sets_name_and_manifest_together() {
    let s = store().await;
    let id = key();
    let created = s
        .create_snapshot(O, id, "look", &manifest_with_exposure(0.5))
        .await
        .unwrap();

    let updated = s
        .update_snapshot(
            O,
            id,
            created.id,
            Some("warmer"),
            Some(&manifest_with_exposure(-1.0)),
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(updated.name, "warmer");
    assert_eq!(updated.edits.basic.exposure_ev, -1.0);
    assert_eq!(updated.created_at, created.created_at);
}

#[tokio::test]
async fn a_rename_keeps_the_edits() {
    let s = store().await;
    let id = key();
    let created = s
        .create_snapshot(O, id, "look", &manifest_with_exposure(0.5))
        .await
        .unwrap();

    let renamed = s
        .update_snapshot(O, id, created.id, Some("renamed"), None)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(renamed.name, "renamed");
    assert_eq!(renamed.manifest_hash, created.manifest_hash);
}

#[tokio::test]
async fn an_update_of_another_owners_snapshot_finds_nothing() {
    let s = store().await;
    let id = key();
    let created = s
        .create_snapshot(O, id, "look", &manifest_with_exposure(0.5))
        .await
        .unwrap();

    let other = s
        .update_snapshot(
            Uuid::new_v4(),
            id,
            created.id,
            Some("stolen"),
            Some(&manifest_with_exposure(2.0)),
        )
        .await
        .unwrap();

    assert!(other.is_none());
    let read = s.get_snapshot(O, id, created.id).await.unwrap().unwrap();
    assert_eq!(read.name, "look");
    assert_eq!(read.manifest_hash, created.manifest_hash);
}

#[tokio::test]
async fn a_manifest_update_clamps_the_edits() {
    let s = store().await;
    let id = key();
    let created = s
        .create_snapshot(O, id, "look", &manifest_with_exposure(0.5))
        .await
        .unwrap();

    let updated = s
        .update_snapshot(O, id, created.id, None, Some(&manifest_with_exposure(99.0)))
        .await
        .unwrap()
        .unwrap();

    let clamped = manifest_with_exposure(99.0).to_edits().clamped();
    assert_eq!(updated.manifest_hash, clamped.stable_hash());
}
