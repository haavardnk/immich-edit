use crate::common::*;
use axum::http::StatusCode;
use immich_edit_backend::asset_key::AssetKey;
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;

const QUERY: &str = "/api/export-outputs/query";

async fn lookup(conflicts: &Conflicts, body: Value) -> (StatusCode, Value) {
    let resp = conflicts
        .app
        .clone()
        .oneshot(json_request("POST", QUERY, body))
        .await
        .unwrap();
    let status = resp.status();
    (status, body_json(resp).await)
}

async fn asset_reads(conflicts: &Conflicts, ids: &[Uuid]) -> usize {
    let mut reads = 0;
    for id in ids {
        reads += conflicts.calls("GET", &format!("/api/assets/{id}")).await;
    }
    reads
}

#[tokio::test]
async fn one_photo_lists_only_its_live_exports_with_the_requested_presets() {
    let conflicts = Conflicts::start(true).await;
    conflicts.previous("web", PreviousExport::Live).await;
    let trashed = conflicts.previous("print", PreviousExport::Trashed).await;
    let gone = conflicts
        .previous("archive", PreviousExport::Answers(404))
        .await;
    let rejected = conflicts
        .previous("square", PreviousExport::Answers(400))
        .await;
    let unasked = conflicts.previous("", PreviousExport::Live).await;

    let (status, json) = lookup(
        &conflicts,
        json!({
            "asset_ids": [master().to_string()],
            "presets": ["Web", "Print", "Archive", "SQUARE", " web "]
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{json}");
    assert_eq!(json["photos"], 1);
    let files = json["files"].as_array().unwrap();
    assert_eq!(files.len(), 1, "{json}");
    assert_eq!(files[0]["filename"], EDIT_NAME);
    let exported_at = files[0]["exported_at"].as_str().unwrap();
    assert!(chrono::DateTime::parse_from_rfc3339(exported_at).is_ok());
    assert_eq!(asset_reads(&conflicts, &[trashed, gone, rejected]).await, 3);
    assert_eq!(asset_reads(&conflicts, &[unasked]).await, 0);
}

#[tokio::test]
async fn one_photo_without_presets_looks_up_plain_exports() {
    let conflicts = Conflicts::start(true).await;
    conflicts.previous("", PreviousExport::Live).await;
    conflicts.previous("web", PreviousExport::Live).await;

    let (status, json) = lookup(
        &conflicts,
        json!({ "asset_ids": [master().to_string()], "presets": [] }),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{json}");
    assert_eq!(json["photos"], 1);
    assert_eq!(json["files"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn one_photo_whose_exports_are_all_gone_counts_none() {
    let conflicts = Conflicts::start(true).await;
    conflicts.previous("", PreviousExport::Trashed).await;
    conflicts
        .previous("web", PreviousExport::Answers(404))
        .await;

    for presets in [json!([]), json!(["Web"]), json!(["Print"])] {
        let (status, json) = lookup(
            &conflicts,
            json!({ "asset_ids": [master().to_string()], "presets": presets }),
        )
        .await;

        assert_eq!(status, StatusCode::OK, "{json}");
        assert_eq!(json, json!({ "photos": 0, "files": [] }), "{presets}");
    }
}

#[tokio::test]
async fn an_immich_failure_on_a_previous_export_is_reported() {
    let conflicts = Conflicts::start(true).await;
    conflicts.previous("", PreviousExport::Answers(500)).await;

    let (status, json) = lookup(&conflicts, json!({ "asset_ids": [master().to_string()] })).await;

    assert_eq!(status, StatusCode::BAD_GATEWAY, "{json}");
}

#[tokio::test]
async fn several_photos_are_counted_from_the_records_alone() {
    let conflicts = Conflicts::start(true).await;
    let [plain, web, both, untracked] = [(); 4].map(|_| AssetKey::master(Uuid::new_v4()));
    let tracked = [(); 4].map(|_| Uuid::new_v4());
    conflicts.track(plain, "", tracked[0]).await;
    conflicts.track(web, "web", tracked[1]).await;
    conflicts.track(both, "", tracked[2]).await;
    conflicts.track(both, "web", tracked[3]).await;
    let asset_ids: Vec<String> = [plain, web, both, untracked, plain]
        .iter()
        .map(AssetKey::to_string)
        .collect();

    for (presets, photos) in [(json!([]), 2), (json!(["Web"]), 2), (json!(["Print"]), 0)] {
        let (status, json) = lookup(
            &conflicts,
            json!({ "asset_ids": asset_ids, "presets": presets }),
        )
        .await;

        assert_eq!(status, StatusCode::OK, "{json}");
        assert_eq!(json, json!({ "photos": photos, "files": [] }), "{presets}");
    }
    assert_eq!(asset_reads(&conflicts, &tracked).await, 0);
}

#[tokio::test]
async fn ten_thousand_photos_are_counted_in_one_lookup() {
    let conflicts = Conflicts::start(true).await;
    let tracked = AssetKey::master(Uuid::new_v4());
    conflicts.track(tracked, "", Uuid::new_v4()).await;
    let mut asset_ids: Vec<String> = (1..10_000).map(|_| Uuid::new_v4().to_string()).collect();
    asset_ids.push(tracked.to_string());

    let (status, json) = lookup(&conflicts, json!({ "asset_ids": asset_ids })).await;

    assert_eq!(status, StatusCode::OK, "{json}");
    assert_eq!(json["photos"], 1);
}

#[tokio::test]
async fn a_lookup_refuses_bad_ids_and_too_many_presets() {
    let conflicts = Conflicts::start(true).await;
    let one = [master().to_string()];
    let presets: Vec<String> = (0..17).map(|i| format!("P{i}")).collect();
    let too_many: Vec<String> = (0..10_001).map(|_| Uuid::new_v4().to_string()).collect();

    for body in [
        json!({ "asset_ids": ["not-a-uuid"] }),
        json!({ "asset_ids": [format!("{}_01", asset_id())] }),
        json!({ "asset_ids": [] }),
        json!({ "asset_ids": too_many }),
        json!({ "asset_ids": one, "presets": presets }),
        json!({ "asset_ids": one, "presets": [""] }),
    ] {
        let (status, json) = lookup(&conflicts, body).await;

        assert_eq!(status, StatusCode::BAD_REQUEST, "{json}");
    }
}

#[tokio::test]
async fn a_lookup_sees_only_the_callers_exports() {
    let server = wiremock::MockServer::start().await;
    let (state, owner, member) = two_owner_state(&server).await;
    state
        .edits
        .put_export_output(output_key(master(), "").output(Uuid::new_v4(), None))
        .await
        .unwrap();
    let other = AssetKey::master(Uuid::new_v4());
    let body = json!({ "asset_ids": [master().to_string(), other.to_string()] });

    for (app, photos) in [(owner, 1), (member, 0)] {
        let resp = app
            .oneshot(json_request("POST", QUERY, body.clone()))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(body_json(resp).await["photos"], photos);
    }
}
