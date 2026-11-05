use super::*;

#[tokio::test]
async fn restoring_a_snapshot_writes_one_history_step() {
    let server = MockServer::start().await;
    mock_asset_detail(&server).await;
    let app = test_app(&server).await;
    let id = asset_id();
    let key = id.to_string();
    put_edits(&app, id, exposure_manifest(0.5)).await;
    let snapshot = create(&app, &key, "Bright", exposure_manifest(1.5)).await;
    assert_eq!(history(&app, &key).await.len(), 1);

    let resp = send(&app, restore_request(&key, &id_of(&snapshot), None)).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let restored = body_json(resp).await;
    assert_eq!(restored["manifest"]["ops"]["exposure"]["ev"], 1.5);
    assert_eq!(restored["hash"], snapshot["manifest_hash"]);
    assert_eq!(restored["immich_checksum"], "abc");

    let entries = history(&app, &key).await;
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0]["action"], "Restore snapshot: Bright");
    assert_eq!(entries[0]["manifest_hash"], snapshot["manifest_hash"]);
    assert_eq!(entries[0]["manifest"]["ops"]["exposure"]["ev"], 1.5);
    let current = body_json(send(&app, get(&format!("/api/assets/{key}/edits"))).await).await;
    assert_eq!(current["manifest"]["ops"]["exposure"]["ev"], 1.5);
}

#[tokio::test]
async fn a_snapshot_restore_after_an_undo_discards_the_redo_branch() {
    let server = MockServer::start().await;
    mock_asset_detail(&server).await;
    let app = test_app(&server).await;
    let id = asset_id();
    let key = id.to_string();
    let a = put_edits(&app, id, exposure_manifest(0.5)).await;
    let b = put_edits(&app, id, exposure_manifest(1.0)).await;
    let snapshot = create(&app, &key, "Shadows", exposure_manifest(-1.0)).await;
    let a_id = history_entry_id(&app, &key, &a["hash"]).await;
    assert_eq!(
        restore_history(&app, &key, a_id).await.status(),
        StatusCode::OK
    );
    let undone = history(&app, &key).await;
    assert_eq!(undone[0]["manifest_hash"], b["hash"]);
    assert_eq!(undone[0]["undone"], true);

    let resp = send(&app, restore_request(&key, &id_of(&snapshot), None)).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let entries = history(&app, &key).await;
    let hashes: Vec<&Value> = entries.iter().map(|e| &e["manifest_hash"]).collect();
    assert_eq!(hashes, [&snapshot["manifest_hash"], &a["hash"]]);
    assert_eq!(entries[0]["action"], "Restore snapshot: Shadows");
    assert!(entries.iter().all(|e| e["undone"] == false), "{entries:?}");
}

#[tokio::test]
async fn restoring_a_snapshot_of_the_current_edits_writes_no_step() {
    let server = MockServer::start().await;
    mock_asset_detail(&server).await;
    let app = test_app(&server).await;
    let id = asset_id();
    let key = id.to_string();
    let current = put_edits(&app, id, exposure_manifest(0.5)).await;
    let snapshot = create(&app, &key, "Same", exposure_manifest(0.5)).await;
    assert_eq!(snapshot["manifest_hash"], current["hash"]);

    let resp = send(&app, restore_request(&key, &id_of(&snapshot), None)).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_json(resp).await["hash"], current["hash"]);

    let entries = history(&app, &key).await;
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["manifest_hash"], current["hash"]);
    assert!(entries[0]["action"].as_str() != Some("Restore snapshot: Same"));
}

#[tokio::test]
async fn a_stale_if_match_restore_conflicts_with_the_current_record() {
    let server = MockServer::start().await;
    mock_asset_detail(&server).await;
    let app = test_app(&server).await;
    let id = asset_id();
    let key = id.to_string();
    let stale = put_edits(&app, id, exposure_manifest(0.5)).await;
    let snapshot = create(&app, &key, "Dark", exposure_manifest(-2.0)).await;
    let current = put_edits(&app, id, exposure_manifest(1.0)).await;
    let stale_hash = stale["hash"].as_str().unwrap();
    let current_hash = current["hash"].as_str().unwrap();

    let resp = send(
        &app,
        restore_request(&key, &id_of(&snapshot), Some(stale_hash)),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
    let conflict = body_json(resp).await;
    assert_eq!(conflict["hash"], current_hash);
    assert_eq!(conflict["manifest"]["ops"]["exposure"]["ev"], 1.0);
    assert_eq!(history(&app, &key).await.len(), 2);

    let resp = send(
        &app,
        restore_request(&key, &id_of(&snapshot), Some(current_hash)),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        body_json(resp).await["manifest"]["ops"]["exposure"]["ev"],
        -2.0
    );
    assert_eq!(history(&app, &key).await.len(), 3);
}
