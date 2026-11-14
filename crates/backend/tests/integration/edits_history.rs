use crate::common::*;
use axum::http::StatusCode;
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::MockServer;

fn timeline(entries: &[Value]) -> Vec<(Value, Value)> {
    entries
        .iter()
        .map(|e| {
            (
                e["manifest"]["ops"]["exposure"]["ev"].clone(),
                e["undone"].clone(),
            )
        })
        .collect()
}

fn step(ev: Value, undone: bool) -> (Value, Value) {
    (ev, json!(undone))
}

async fn current_exposure(app: &axum::Router, id: Uuid) -> Value {
    let current = body_json(
        app.clone()
            .oneshot(get(&format!("/api/assets/{id}/edits")))
            .await
            .unwrap(),
    )
    .await;
    current["manifest"]["ops"]["exposure"]["ev"].clone()
}

#[tokio::test]
async fn put_writes_history_revision() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_asset_detail(&server).await;
    let app = test_app(&server).await;

    put_edits(&app, id, exposure_manifest(0.5)).await;
    put_edits(&app, id, exposure_manifest(1.5)).await;

    let entries = history(&app, id).await;
    if timeline(&entries) != [step(json!(1.5), false), step(json!(0.5), false)] {
        panic!("expected newest first, none undone: {entries:?}");
    }
}

#[tokio::test]
async fn unchanged_put_keeps_record_and_history() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_asset_detail(&server).await;
    let app = test_app(&server).await;

    let first = put_edits(&app, id, exposure_manifest(0.5)).await;
    let again = put_edits(&app, id, exposure_manifest(0.5)).await;
    if again["updated_at"] != first["updated_at"] || again["hash"] != first["hash"] {
        panic!("unchanged put rewrote the record: {first} -> {again}");
    }

    let entries = history(&app, id).await;
    if entries.len() != 1 {
        panic!("expected 1 history entry: {entries:?}");
    }
}

#[tokio::test]
async fn restore_returns_previous_edits() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_asset_detail(&server).await;
    let app = test_app(&server).await;

    let first = put_edits(&app, id, exposure_manifest(0.5)).await;
    put_edits(&app, id, exposure_manifest(1.5)).await;
    let entry_id = history_entry_id(&app, id, &first["hash"]).await;

    let resp = restore_history(&app, id, entry_id).await;
    if resp.status() != StatusCode::OK {
        panic!("restore status {}", resp.status());
    }
    let restored = body_json(resp).await;
    if restored["manifest"]["ops"]["exposure"]["ev"] != 0.5 {
        panic!("restore body: {restored}");
    }
    if current_exposure(&app, id).await != 0.5 {
        panic!("get-after-restore did not return the restored edits");
    }
}

#[tokio::test]
async fn restore_after_reset_refreshes_upstream_meta() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_asset_detail(&server).await;
    let app = test_app(&server).await;

    let saved = put_edits(&app, id, exposure_manifest(0.5)).await;

    let resp = app
        .clone()
        .oneshot(empty_request("DELETE", &format!("/api/assets/{id}/edits")))
        .await
        .unwrap();
    if resp.status() != StatusCode::NO_CONTENT {
        panic!("delete status {}", resp.status());
    }

    let entry_id = history_entry_id(&app, id, &saved["hash"]).await;
    let resp = restore_history(&app, id, entry_id).await;
    if resp.status() != StatusCode::OK {
        panic!("restore status {}", resp.status());
    }
    let restored = body_json(resp).await;
    if restored["immich_checksum"] != "abc" || restored["immich_updated_at"].is_null() {
        panic!("restore meta: {restored}");
    }
}

#[tokio::test]
async fn restore_keeps_later_entries_so_it_can_be_undone() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_asset_detail(&server).await;
    let app = test_app(&server).await;

    let first = put_edits(&app, id, exposure_manifest(0.5)).await;
    let second = put_edits(&app, id, exposure_manifest(1.0)).await;
    put_edits(&app, id, exposure_manifest(1.5)).await;
    let first_id = history_entry_id(&app, id, &first["hash"]).await;
    let second_id = history_entry_id(&app, id, &second["hash"]).await;

    let resp = restore_history(&app, id, first_id).await;
    if resp.status() != StatusCode::OK {
        panic!("restore status {}", resp.status());
    }
    let entries = history(&app, id).await;
    let expected = [
        step(json!(1.5), true),
        step(json!(1.0), true),
        step(json!(0.5), false),
    ];
    if timeline(&entries) != expected {
        panic!("later entries not kept as undone: {entries:?}");
    }

    let resp = restore_history(&app, id, second_id).await;
    if resp.status() != StatusCode::OK {
        panic!("forward restore status {}", resp.status());
    }
    let restored = body_json(resp).await;
    if restored["manifest"]["ops"]["exposure"]["ev"] != 1.0 {
        panic!("forward restore body: {restored}");
    }
    if current_exposure(&app, id).await != 1.0 {
        panic!("forward restore did not bring the edits back");
    }
    let entries = history(&app, id).await;
    let expected = [
        step(json!(1.5), true),
        step(json!(1.0), false),
        step(json!(0.5), false),
    ];
    if timeline(&entries) != expected {
        panic!("flags not updated by the forward restore: {entries:?}");
    }
}

#[tokio::test]
async fn a_new_edit_discards_undone_entries() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_asset_detail(&server).await;
    let app = test_app(&server).await;

    let first = put_edits(&app, id, exposure_manifest(0.5)).await;
    put_edits(&app, id, exposure_manifest(1.0)).await;
    let first_id = history_entry_id(&app, id, &first["hash"]).await;
    let resp = restore_history(&app, id, first_id).await;
    if resp.status() != StatusCode::OK {
        panic!("restore status {}", resp.status());
    }

    put_edits(&app, id, exposure_manifest(2.0)).await;
    let entries = history(&app, id).await;
    if timeline(&entries) != [step(json!(2.0), false), step(json!(0.5), false)] {
        panic!("undone entries survived a new edit: {entries:?}");
    }
}

#[tokio::test]
async fn restoring_a_reset_answers_no_content_and_keeps_later_entries() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_asset_detail(&server).await;
    let app = test_app(&server).await;

    put_edits(&app, id, exposure_manifest(0.5)).await;
    let resp = app
        .clone()
        .oneshot(empty_request("DELETE", &format!("/api/assets/{id}/edits")))
        .await
        .unwrap();
    if resp.status() != StatusCode::NO_CONTENT {
        panic!("delete status {}", resp.status());
    }
    put_edits(&app, id, exposure_manifest(1.0)).await;
    let reset_id = history(&app, id)
        .await
        .iter()
        .find(|e| e["deleted"] == true)
        .and_then(|e| e["id"].as_i64())
        .expect("reset entry");

    let resp = restore_history(&app, id, reset_id).await;
    if resp.status() != StatusCode::NO_CONTENT {
        panic!("restore reset status {}", resp.status());
    }
    if !current_exposure(&app, id).await.is_null() {
        panic!("edits kept after restoring a reset");
    }
    let entries = history(&app, id).await;
    let expected = [
        step(json!(1.0), true),
        step(Value::Null, false),
        step(json!(0.5), false),
    ];
    if timeline(&entries) != expected {
        panic!("later entry not kept as undone: {entries:?}");
    }
}

#[tokio::test]
async fn restoring_an_unknown_entry_is_not_found() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_asset_detail(&server).await;
    let app = test_app(&server).await;

    put_edits(&app, id, exposure_manifest(0.5)).await;
    let known = history(&app, id).await[0]["id"].as_i64().unwrap();

    let resp = restore_history(&app, id, known + 1000).await;
    if resp.status() != StatusCode::NOT_FOUND {
        panic!("unknown entry status {}", resp.status());
    }
    if current_exposure(&app, id).await != 0.5 {
        panic!("a failed restore changed the edits");
    }
}
