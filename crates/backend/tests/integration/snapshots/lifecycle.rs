use super::*;
use immich_edit_backend::asset_key::AssetKey;
use immich_edit_backend::state::AppState;
use raw_pipeline::edit_manifest::EditManifest;
use raw_pipeline::edits::Edits;

fn depth_manifest(raster_id: &str) -> Value {
    let mut edits = Edits::default();
    edits.lens_blur.depth_raster_id = raster_id.into();
    serde_json::to_value(EditManifest::from_edits(&edits)).unwrap()
}

async fn pinned_for(state: &AppState, owner: Uuid, key: &str) -> Vec<String> {
    sqlx::query_scalar("SELECT raster_id FROM raster_refs WHERE user_id = ?1 AND asset_id = ?2")
        .bind(owner.to_string())
        .bind(key)
        .fetch_all(&state.edits.pool())
        .await
        .unwrap()
}

async fn pinned(state: &AppState, key: &str) -> Vec<String> {
    pinned_for(state, test_user_id(), key).await
}

#[tokio::test]
async fn a_snapshot_pins_its_rasters_after_the_edits_drop_them() {
    let server = MockServer::start().await;
    mock_asset_detail(&server).await;
    let state = test_state(&server).await;
    let app = seed_and_wrap(&server, state.clone()).await;
    let id = asset_id();
    let key = id.to_string();
    let first = put_edits(&app, id, exposure_manifest(0.5)).await;
    put_edits(&app, id, depth_manifest("depth-r")).await;
    let snapshot = create(&app, &key, "Depth", depth_manifest("depth-r")).await;
    let first_id = history_entry_id(&app, &key, &first["hash"]).await;
    assert_eq!(
        restore_history(&app, &key, first_id).await.status(),
        StatusCode::OK
    );
    put_edits(&app, id, exposure_manifest(2.0)).await;
    assert_eq!(history(&app, &key).await.len(), 2);

    assert_eq!(pinned(&state, &key).await, ["depth-r"]);
    state.edits.rebuild_raster_refs().await.unwrap();
    assert_eq!(pinned(&state, &key).await, ["depth-r"]);

    let resp = send(
        &app,
        empty_request("DELETE", &snapshot_uri(&key, &id_of(&snapshot))),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    assert!(pinned(&state, &key).await.is_empty());
}

#[tokio::test]
async fn deleting_a_snapshot_keeps_rasters_the_history_still_uses() {
    let server = MockServer::start().await;
    mock_asset_detail(&server).await;
    let state = test_state(&server).await;
    let app = seed_and_wrap(&server, state.clone()).await;
    let id = asset_id();
    let key = id.to_string();
    put_edits(&app, id, depth_manifest("depth-r")).await;
    let snapshot = create(&app, &key, "Depth", depth_manifest("depth-r")).await;
    put_edits(&app, id, exposure_manifest(1.0)).await;

    let resp = send(
        &app,
        empty_request("DELETE", &snapshot_uri(&key, &id_of(&snapshot))),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    assert_eq!(pinned(&state, &key).await, ["depth-r"]);
}

#[tokio::test]
async fn deleting_a_copy_removes_its_snapshots() {
    let server = MockServer::start().await;
    mock_asset_detail(&server).await;
    let app = test_app(&server).await;
    let key = asset_id().to_string();
    let resp = send(
        &app,
        json_request("POST", &format!("/api/assets/{key}/copies"), json!({})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let copy = body_json(resp).await["id"].as_str().unwrap().to_string();
    create(&app, &copy, "Copy look", exposure_manifest(1.0)).await;
    create(&app, &key, "Master look", exposure_manifest(0.5)).await;

    let resp = send(
        &app,
        empty_request("DELETE", &format!("/api/copies/{copy}")),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    assert!(list(&app, &copy).await.is_empty());
    assert_eq!(names(&list(&app, &key).await), ["Master look"]);
}

async fn snapshot_count(state: &AppState, owner: Uuid) -> usize {
    state
        .edits
        .list_snapshots(owner, AssetKey::master(asset_id()))
        .await
        .unwrap()
        .len()
}

#[tokio::test]
async fn purging_an_owner_removes_only_their_snapshots() {
    let server = MockServer::start().await;
    let (state, admin, member) = two_owner_state(&server).await;
    let key = asset_id().to_string();
    create(&admin, &key, "Admin look", depth_manifest("admin-r")).await;
    create(&member, &key, "Member look", depth_manifest("member-r")).await;
    assert_eq!(
        pinned_for(&state, member_user_id(), &key).await,
        ["member-r"]
    );

    state.edits.purge_owner(member_user_id()).await.unwrap();

    assert_eq!(snapshot_count(&state, member_user_id()).await, 0);
    assert_eq!(snapshot_count(&state, test_user_id()).await, 1);
    assert!(pinned_for(&state, member_user_id(), &key).await.is_empty());
    assert_eq!(pinned_for(&state, test_user_id(), &key).await, ["admin-r"]);
}

#[tokio::test]
async fn a_rebind_removes_all_snapshots() {
    let server = MockServer::start().await;
    let (state, admin, member) = two_owner_state(&server).await;
    let key = asset_id().to_string();
    create(&admin, &key, "Admin look", exposure_manifest(0.5)).await;
    create(&member, &key, "Member look", exposure_manifest(1.0)).await;

    rebind_instance(&server, &state).await;

    for owner in [test_user_id(), member_user_id()] {
        assert_eq!(snapshot_count(&state, owner).await, 0);
    }
}
