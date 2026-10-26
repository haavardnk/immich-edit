use crate::common::*;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use immich_edit_backend::asset_key::AssetKey;
use immich_edit_backend::routes::export::{
    ExportParams, ExportToImmichBody, StackPrimary, hash_request,
};
use immich_edit_backend::services::edits_store::ExportJobKey;
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::matchers::{body_json as body_is, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

struct Replace {
    server: MockServer,
    uploaded: Uuid,
}

fn original(owner: Uuid) -> Value {
    json!({
        "id": asset_id(),
        "originalFileName": "IMG_0001.ARW",
        "type": "IMAGE",
        "ownerId": owner
    })
}

impl Replace {
    async fn start(owner: Uuid, trash: bool) -> Self {
        Self::with(original(owner), json!({}), trash).await
    }

    async fn with(original: Value, edit: Value, trash: bool) -> Self {
        let server = MockServer::start().await;
        let uploaded = edit["id"]
            .as_str()
            .map_or_else(Uuid::new_v4, |id| id.parse().unwrap());
        let mut edit_detail = json!({
            "id": uploaded,
            "originalFileName": "IMG_0001_edit.jpg",
            "type": "IMAGE",
            "width": 6000
        });
        edit_detail
            .as_object_mut()
            .unwrap()
            .extend(edit.as_object().cloned().unwrap_or_default());
        for detail in [original, edit_detail] {
            Mock::given(method("GET"))
                .and(path(format!(
                    "/api/assets/{}",
                    detail["id"].as_str().unwrap()
                )))
                .respond_with(ResponseTemplate::new(200).set_body_json(detail))
                .mount(&server)
                .await;
        }
        mock_trash_feature(&server, trash).await;
        Self { server, uploaded }
    }

    async fn expect_copy(
        &self,
        favorite: bool,
        stack: bool,
        response: ResponseTemplate,
        times: u64,
    ) {
        Mock::given(method("PUT"))
            .and(path("/api/assets/copy"))
            .and(body_is(json!({
                "sourceId": asset_id(),
                "targetId": self.uploaded,
                "sidecar": false,
                "albums": true,
                "sharedLinks": true,
                "favorite": favorite,
                "stack": stack,
            })))
            .respond_with(response)
            .expect(times)
            .mount(&self.server)
            .await;
    }

    async fn expect_trash(&self, times: u64) {
        Mock::given(method("DELETE"))
            .and(path("/api/assets"))
            .and(body_is(json!({ "ids": [asset_id()], "force": false })))
            .respond_with(ResponseTemplate::new(204))
            .expect(times)
            .mount(&self.server)
            .await;
    }

    async fn expect_copy_and_trash(&self, times: u64) {
        self.expect_copy(true, true, ResponseTemplate::new(204), times)
            .await;
        self.expect_trash(times).await;
    }

    async fn album_answers(&self, album: Uuid, error: &str) {
        Mock::given(method("PUT"))
            .and(path(format!("/api/albums/{album}/assets")))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                { "id": self.uploaded, "success": false, "error": error }
            ])))
            .mount(&self.server)
            .await;
    }

    async fn resume(&self, request: Value) -> (StatusCode, Value) {
        let state = test_state(&self.server).await;
        let asset = AssetKey::master(asset_id());
        let body: ExportToImmichBody = serde_json::from_value(request.clone()).unwrap();
        let job = ExportJobKey {
            owner: test_user_id(),
            asset_id: asset,
            key: "key-1",
        };
        state
            .edits
            .put_export_job_uploaded(
                job,
                &hash_request(asset, &body),
                self.uploaded,
                "IMG_0001_edit.jpg",
                "created",
                &[],
            )
            .await
            .unwrap();
        let app = seed_and_wrap(&self.server, state).await;
        let first = post_export(&app, &request).await;
        let replay = post_export(&app, &request).await;
        if replay != first {
            panic!("replay differs: {first:?} vs {replay:?}");
        }
        first
    }
}

async fn post_export(app: &axum::Router, request: &Value) -> (StatusCode, Value) {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/assets/{}/export/immich", asset_id()))
                .header("content-type", "application/json")
                .header("idempotency-key", "key-1")
                .body(Body::from(request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    (status, body_json(resp).await)
}

fn warnings(json: &Value) -> Vec<String> {
    json["warnings"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|w| w.as_str().map(str::to_string))
        .collect()
}

fn assert_kept(status: StatusCode, json: &Value, reason: &str) {
    if status != StatusCode::OK
        || json["original_trashed"] != false
        || warnings(json).last().map(String::as_str) != Some(reason)
    {
        panic!("{reason}: {status} {json}");
    }
}

#[tokio::test]
async fn replace_original_moves_links_then_trashes_the_original() {
    let replace = Replace::start(test_user_id(), true).await;
    replace.expect_copy_and_trash(1).await;

    let (status, json) = replace.resume(json!({ "replace_original": true })).await;

    if status != StatusCode::OK || json["original_trashed"] != true || !warnings(&json).is_empty() {
        panic!("replace: {status} {json}");
    }
}

#[tokio::test]
async fn replace_original_keeps_a_favorite_asked_for_on_upload() {
    let replace = Replace::start(test_user_id(), true).await;
    replace
        .expect_copy(false, true, ResponseTemplate::new(204), 1)
        .await;
    replace.expect_trash(1).await;

    let (status, json) = replace
        .resume(json!({ "replace_original": true, "favorite": true }))
        .await;

    if status != StatusCode::OK || json["original_trashed"] != true {
        panic!("favorite: {status} {json}");
    }
}

#[tokio::test]
async fn replace_original_keeps_the_original_when_a_step_fails() {
    for (error, trashed) in [("duplicate", true), ("no_permission", false)] {
        let replace = Replace::start(test_user_id(), true).await;
        let album = Uuid::new_v4();
        replace.album_answers(album, error).await;
        replace.expect_copy_and_trash(u64::from(trashed)).await;

        let (status, json) = replace
            .resume(json!({ "replace_original": true, "album_ids": [album] }))
            .await;

        let kept = warnings(&json)
            .iter()
            .any(|w| w == "Original kept: an earlier step failed");
        if status != StatusCode::OK || json["original_trashed"] != trashed || kept == trashed {
            panic!("album {error}: {status} {json}");
        }
    }
}

#[tokio::test]
async fn replace_original_skips_photos_of_other_users_and_a_disabled_trash() {
    for (owner, trash, reason) in [
        (
            Uuid::new_v4(),
            true,
            "Original kept: it belongs to another Immich user",
        ),
        (
            test_user_id(),
            false,
            "Original kept: the Immich trash is turned off",
        ),
    ] {
        let replace = Replace::start(owner, trash).await;
        replace.expect_copy_and_trash(0).await;

        let (status, json) = replace.resume(json!({ "replace_original": true })).await;

        assert_kept(status, &json, reason);
    }
}

#[tokio::test]
async fn replace_original_never_trashes_the_only_copy() {
    for (edit, reason) in [
        (
            json!({ "id": asset_id() }),
            "Original kept: Immich matched the upload to the original",
        ),
        (
            json!({ "isTrashed": true }),
            "Original kept: the matching upload is in the Immich trash",
        ),
    ] {
        let replace = Replace::with(original(test_user_id()), edit, true).await;
        replace.expect_copy_and_trash(0).await;

        let (status, json) = replace.resume(json!({ "replace_original": true })).await;

        assert_kept(status, &json, reason);
    }
}

#[tokio::test]
async fn replace_original_reports_a_refused_copy() {
    let replace = Replace::start(test_user_id(), true).await;
    replace
        .expect_copy(
            true,
            true,
            ResponseTemplate::new(403)
                .set_body_json(json!({ "message": "Missing required permission: asset.copy" })),
            1,
        )
        .await;
    replace.expect_trash(0).await;

    let (status, json) = replace.resume(json!({ "replace_original": true })).await;

    assert_kept(
        status,
        &json,
        "Original kept: copying its albums and links failed: Missing required permission: asset.copy",
    );
}

#[tokio::test]
async fn replace_original_puts_the_upload_on_top_of_the_stack() {
    let stack = Uuid::new_v4();
    let mut stacked = original(test_user_id());
    stacked["stack"] = json!({ "id": stack, "primaryAssetId": asset_id(), "assetCount": 2 });
    let replace = Replace::with(stacked, json!({}), true).await;
    replace.expect_copy_and_trash(1).await;
    Mock::given(method("PATCH"))
        .and(path(format!("/api/stacks/{stack}")))
        .and(body_is(json!({ "primaryAssetId": replace.uploaded })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": stack,
            "primaryAssetId": replace.uploaded,
            "assets": []
        })))
        .expect(1)
        .mount(&replace.server)
        .await;

    let (status, json) = replace.resume(json!({ "replace_original": true })).await;

    if status != StatusCode::OK || json["original_trashed"] != true {
        panic!("stack primary: {status} {json}");
    }
}

#[tokio::test]
async fn replace_original_does_not_restack_an_edit_already_in_the_stack() {
    let stack = Uuid::new_v4();
    let mut stacked = original(test_user_id());
    stacked["stackId"] = json!(stack);
    let replace = Replace::with(stacked, json!({ "stackId": stack }), true).await;
    replace
        .expect_copy(true, false, ResponseTemplate::new(204), 1)
        .await;
    replace.expect_trash(1).await;

    let (status, json) = replace.resume(json!({ "replace_original": true })).await;

    if status != StatusCode::OK || json["original_trashed"] != true {
        panic!("restack: {status} {json}");
    }
}

#[tokio::test]
async fn replace_original_is_refused_up_front() {
    let replace = Replace::start(test_user_id(), false).await;
    replace.expect_copy_and_trash(0).await;
    let app = test_app(&replace.server).await;

    let (status, json) = post_export(
        &app,
        &json!({ "replace_original": true, "stack_with_original": true }),
    )
    .await;
    if status != StatusCode::BAD_REQUEST {
        panic!("with stacking: {status} {json}");
    }

    let (status, json) = post_export(&app, &json!({ "replace_original": true })).await;
    if status != StatusCode::CONFLICT || json["code"] != "trash_disabled" {
        panic!("trash off: {status} {json}");
    }
}

#[tokio::test]
async fn replace_original_jobs_take_one_version_per_photo() {
    let id = asset_id();
    for (trash, params, ids, want) in [
        (
            true,
            json!({ "replace_original": true }),
            json!([id, format!("{id}_1")]),
            StatusCode::BAD_REQUEST,
        ),
        (
            true,
            json!({ "replace_original": true, "stack_with_original": true }),
            json!([id]),
            StatusCode::BAD_REQUEST,
        ),
        (
            false,
            json!({ "replace_original": true }),
            json!([id]),
            StatusCode::CONFLICT,
        ),
        (
            true,
            json!({ "replace_original": true }),
            json!([id]),
            StatusCode::OK,
        ),
    ] {
        let replace = Replace::start(test_user_id(), trash).await;
        let resp = test_app(&replace.server)
            .await
            .oneshot(json_request(
                "POST",
                "/api/jobs",
                json!({ "kind": "export_immich", "asset_ids": ids, "params": params }),
            ))
            .await
            .unwrap();
        if resp.status() != want {
            panic!("{params} {ids}: {}", resp.status());
        }
    }
}

#[test]
fn hash_request_differs_by_replace_original() {
    let asset = AssetKey::master(Uuid::nil());
    let make = |replace_original| ExportToImmichBody {
        edits: Default::default(),
        params: ExportParams::default(),
        album_ids: Vec::new(),
        tag_ids: Vec::new(),
        favorite: false,
        stack_with_original: false,
        stack_primary: StackPrimary::default(),
        replace_original,
    };
    if hash_request(asset, &make(true)) == hash_request(asset, &make(false)) {
        panic!("replace_original must change the request hash");
    }
}
