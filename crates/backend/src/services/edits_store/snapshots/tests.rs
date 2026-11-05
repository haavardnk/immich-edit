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
