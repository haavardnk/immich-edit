mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::*;
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn get(uri: &str) -> Request<Body> {
    Request::builder().uri(uri).body(Body::empty()).unwrap()
}

fn json_req(verb: &str, uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(verb)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn empty_req(verb: &str, uri: &str) -> Request<Body> {
    Request::builder()
        .method(verb)
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

async fn json_body(resp: axum::response::Response) -> Value {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

async fn send(app: &axum::Router, req: Request<Body>) -> axum::response::Response {
    app.clone().oneshot(req).await.unwrap()
}

async fn expect_status(
    app: &axum::Router,
    req: Request<Body>,
    want: StatusCode,
    what: &str,
) -> axum::response::Response {
    let resp = send(app, req).await;
    if resp.status() != want {
        panic!("{what}: expected {want}, got {}", resp.status());
    }
    resp
}

async fn mock_asset_for_admin(server: &MockServer, id: uuid::Uuid) {
    Mock::given(method("GET"))
        .and(path(format!("/api/assets/{id}")))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": id,
            "originalFileName": "DSC0001.ARW",
            "type": "IMAGE",
            "updatedAt": "2026-05-01T00:00:00Z",
            "checksum": "deadbeef"
        })))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/api/assets/{id}")))
        .and(header("x-api-key", MEMBER_API_KEY))
        .respond_with(ResponseTemplate::new(403))
        .mount(server)
        .await;
}

fn manifest_body(ev: f64) -> Value {
    serde_json::json!({
        "schema_version": 2,
        "ops": { "exposure": { "ev": ev } }
    })
}

async fn save_admin_edits(admin: &axum::Router, id: uuid::Uuid) -> String {
    let saved = json_body(
        expect_status(
            admin,
            json_req(
                "PUT",
                &format!("/api/assets/{id}/edits"),
                manifest_body(1.5),
            ),
            StatusCode::OK,
            "owner put edits",
        )
        .await,
    )
    .await;
    saved["hash"].as_str().expect("edit hash").to_string()
}

#[tokio::test]
async fn edits_and_history_are_scoped_to_the_owner() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_asset_for_admin(&server, id).await;
    let (admin, member) = two_owner_apps(&server).await;
    save_admin_edits(&admin, id).await;

    let owner_list =
        json_body(expect_status(&admin, get("/api/edits"), StatusCode::OK, "owner list").await)
            .await;
    if owner_list.as_array().map(Vec::len) != Some(1) {
        panic!("owner edit list: {owner_list}");
    }

    let record = json_body(
        expect_status(
            &member,
            get(&format!("/api/assets/{id}/edits")),
            StatusCode::OK,
            "member get edits",
        )
        .await,
    )
    .await;
    if !record["manifest"]["ops"]["exposure"].is_null() || record["updated_at"] != "" {
        panic!("member read the owner's edits: {record}");
    }

    let history = json_body(
        expect_status(
            &member,
            get(&format!("/api/assets/{id}/edits/history")),
            StatusCode::OK,
            "member get history",
        )
        .await,
    )
    .await;
    if history.as_array().map(Vec::len) != Some(0) {
        panic!("member read the owner's history: {history}");
    }

    let member_list =
        json_body(expect_status(&member, get("/api/edits"), StatusCode::OK, "member list").await)
            .await;
    if member_list.as_array().map(Vec::len) != Some(0) {
        panic!("member list: {member_list}");
    }
}

#[tokio::test]
async fn a_member_cannot_overwrite_another_owners_edits() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_asset_for_admin(&server, id).await;
    let (admin, member) = two_owner_apps(&server).await;
    save_admin_edits(&admin, id).await;

    // Immich answers the member's asset lookup with 403, which the backend reports as a bad
    // gateway, so the write never reaches the edits table.
    expect_status(
        &member,
        json_req(
            "PUT",
            &format!("/api/assets/{id}/edits"),
            manifest_body(-2.0),
        ),
        StatusCode::BAD_GATEWAY,
        "member put edits",
    )
    .await;
    expect_status(
        &member,
        empty_req("DELETE", &format!("/api/assets/{id}/edits")),
        StatusCode::NO_CONTENT,
        "member delete edits",
    )
    .await;

    let record = json_body(
        expect_status(
            &admin,
            get(&format!("/api/assets/{id}/edits")),
            StatusCode::OK,
            "owner get edits",
        )
        .await,
    )
    .await;
    if record["manifest"]["ops"]["exposure"]["ev"] != 1.5 {
        panic!("member changed the owner's edits: {record}");
    }
}

#[tokio::test]
async fn virtual_copies_are_scoped_to_the_owner() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_asset_for_admin(&server, id).await;
    let (admin, member) = two_owner_apps(&server).await;

    let copy = json_body(
        expect_status(
            &admin,
            json_req(
                "POST",
                &format!("/api/assets/{id}/copies"),
                serde_json::json!({ "name": "owner copy" }),
            ),
            StatusCode::CREATED,
            "owner create copy",
        )
        .await,
    )
    .await;
    let copy_id = copy["id"].as_str().expect("copy id").to_string();

    let listed = json_body(
        expect_status(
            &member,
            get(&format!("/api/assets/{id}/copies")),
            StatusCode::OK,
            "member list copies",
        )
        .await,
    )
    .await;
    if listed.as_array().map(Vec::len) != Some(0) {
        panic!("member listed the owner's copies: {listed}");
    }

    expect_status(
        &member,
        json_req(
            "PATCH",
            &format!("/api/copies/{copy_id}"),
            serde_json::json!({ "name": "stolen" }),
        ),
        StatusCode::NOT_FOUND,
        "member rename copy",
    )
    .await;
    expect_status(
        &member,
        empty_req("DELETE", &format!("/api/copies/{copy_id}")),
        StatusCode::NOT_FOUND,
        "member delete copy",
    )
    .await;

    let still_there = json_body(
        expect_status(
            &admin,
            get(&format!("/api/assets/{id}/copies")),
            StatusCode::OK,
            "owner list copies",
        )
        .await,
    )
    .await;
    if still_there[0]["name"] != "owner copy" {
        panic!("owner copy after member calls: {still_there}");
    }
}

#[tokio::test]
async fn presets_are_scoped_to_the_owner() {
    let server = MockServer::start().await;
    let (admin, member) = two_owner_apps(&server).await;

    let preset = json_body(
        expect_status(
            &admin,
            json_req(
                "POST",
                "/api/presets",
                serde_json::json!({ "name": "Owner look", "manifest": manifest_body(0.5) }),
            ),
            StatusCode::OK,
            "owner create preset",
        )
        .await,
    )
    .await;
    let preset_id = preset["id"].as_str().expect("preset id").to_string();

    let listed =
        json_body(expect_status(&member, get("/api/presets"), StatusCode::OK, "member list").await)
            .await;
    if listed.as_array().map(Vec::len) != Some(0) {
        panic!("member listed the owner's presets: {listed}");
    }

    expect_status(
        &member,
        get(&format!("/api/presets/{preset_id}")),
        StatusCode::NOT_FOUND,
        "member get preset",
    )
    .await;
    expect_status(
        &member,
        json_req(
            "PUT",
            &format!("/api/presets/{preset_id}"),
            serde_json::json!({ "name": "stolen", "manifest": manifest_body(9.0) }),
        ),
        StatusCode::NOT_FOUND,
        "member update preset",
    )
    .await;
    expect_status(
        &member,
        empty_req("DELETE", &format!("/api/presets/{preset_id}")),
        StatusCode::NOT_FOUND,
        "member delete preset",
    )
    .await;

    let owned = json_body(
        expect_status(
            &admin,
            get(&format!("/api/presets/{preset_id}")),
            StatusCode::OK,
            "owner get preset",
        )
        .await,
    )
    .await;
    if owned["name"] != "Owner look" {
        panic!("owner preset after member calls: {owned}");
    }
}

#[tokio::test]
async fn rasters_are_scoped_to_the_owner() {
    let server = MockServer::start().await;
    let (admin, member) = two_owner_apps(&server).await;

    let upload = Request::builder()
        .method("POST")
        .uri("/api/rasters?width=4&height=4")
        .header("content-type", "application/octet-stream")
        .body(Body::from(vec![200u8; 16]))
        .unwrap();
    let meta =
        json_body(expect_status(&admin, upload, StatusCode::OK, "owner upload raster").await).await;
    let raster_id = meta["raster_id"].as_str().expect("raster id").to_string();

    expect_status(
        &member,
        get(&format!("/api/rasters/{raster_id}")),
        StatusCode::NOT_FOUND,
        "member get raster",
    )
    .await;
    expect_status(
        &member,
        get(&format!("/api/rasters/{raster_id}/meta")),
        StatusCode::NOT_FOUND,
        "member get raster meta",
    )
    .await;
    expect_status(
        &admin,
        get(&format!("/api/rasters/{raster_id}")),
        StatusCode::OK,
        "owner get raster",
    )
    .await;
}

fn raster_owner_dir(root: &std::path::Path, raster_id: &str) -> String {
    let file = format!("{raster_id}.r8");
    let dirs = std::fs::read_dir(root)
        .unwrap()
        .flat_map(|epoch| std::fs::read_dir(epoch.unwrap().path()).unwrap())
        .map(|owner| owner.unwrap().path());
    let Some(owner) = dirs.into_iter().find(|dir| dir.join(&file).exists()) else {
        panic!("raster {raster_id} is not on disk");
    };
    owner.file_name().unwrap().to_string_lossy().into_owned()
}

#[tokio::test]
async fn raster_ids_cannot_traverse_into_another_owner() {
    let server = MockServer::start().await;
    let (state, admin, member) = two_owner_state(&server).await;

    let upload = Request::builder()
        .method("POST")
        .uri("/api/rasters?width=4&height=4")
        .header("content-type", "application/octet-stream")
        .body(Body::from(vec![200u8; 16]))
        .unwrap();
    let meta =
        json_body(expect_status(&admin, upload, StatusCode::OK, "owner upload raster").await).await;
    let raster_id = meta["raster_id"].as_str().expect("raster id").to_string();
    let owner = raster_owner_dir(&state.config.data_dir.join("rasters"), &raster_id);

    for suffix in ["", "/meta"] {
        expect_status(
            &member,
            get(&format!("/api/rasters/..%2F{owner}%2F{raster_id}{suffix}")),
            StatusCode::NOT_FOUND,
            "member traverses to owner raster",
        )
        .await;
    }
}

#[tokio::test]
async fn jobs_and_their_downloads_are_scoped_to_the_owner() {
    let server = MockServer::start().await;
    let id = asset_id();
    let (admin, member) = two_owner_apps(&server).await;

    let job = json_body(
        expect_status(
            &admin,
            json_req(
                "POST",
                "/api/jobs",
                serde_json::json!({ "kind": "download_zip", "asset_ids": [id.to_string()] }),
            ),
            StatusCode::OK,
            "owner create job",
        )
        .await,
    )
    .await;
    let job_id = job["id"].as_str().expect("job id").to_string();

    let listed = json_body(
        expect_status(
            &member,
            get("/api/jobs"),
            StatusCode::OK,
            "member list jobs",
        )
        .await,
    )
    .await;
    if listed.as_array().map(Vec::len) != Some(0) {
        panic!("member listed the owner's jobs: {listed}");
    }

    expect_status(
        &member,
        get(&format!("/api/jobs/{job_id}")),
        StatusCode::NOT_FOUND,
        "member get job",
    )
    .await;
    expect_status(
        &member,
        empty_req("POST", &format!("/api/jobs/{job_id}/cancel")),
        StatusCode::NOT_FOUND,
        "member cancel job",
    )
    .await;
    expect_status(
        &member,
        get(&format!("/api/jobs/{job_id}/events")),
        StatusCode::NOT_FOUND,
        "member watch job",
    )
    .await;

    // The owner reaches the kind/status check behind the ownership check, so the member's
    // 404 comes from ownership and not from the job merely being unfinished.
    expect_status(
        &member,
        get(&format!("/api/jobs/{job_id}/download")),
        StatusCode::NOT_FOUND,
        "member download job",
    )
    .await;
    expect_status(
        &admin,
        get(&format!("/api/jobs/{job_id}/download")),
        StatusCode::BAD_REQUEST,
        "owner download job",
    )
    .await;
}

#[tokio::test]
async fn preview_meta_and_scopes_are_scoped_to_the_owner() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_original(&server, id).await;
    let (admin, member) = two_owner_apps(&server).await;

    let resp = expect_status(
        &admin,
        json_req(
            "POST",
            &format!("/api/assets/{id}/preview"),
            serde_json::json!({ "max_edge": 512, "edits": {}, "scopes": true }),
        ),
        StatusCode::OK,
        "owner preview",
    )
    .await;
    let meta_id = resp
        .headers()
        .get("x-preview-meta-id")
        .expect("preview meta id")
        .to_str()
        .unwrap()
        .to_string();

    expect_status(
        &member,
        get(&format!("/api/assets/{id}/preview/meta/{meta_id}")),
        StatusCode::NOT_FOUND,
        "member get preview meta",
    )
    .await;
    expect_status(
        &member,
        get(&format!(
            "/api/assets/{id}/preview/meta/{meta_id}/scope/waveform"
        )),
        StatusCode::NOT_FOUND,
        "member get preview scope",
    )
    .await;
    expect_status(
        &admin,
        get(&format!("/api/assets/{id}/preview/meta/{meta_id}")),
        StatusCode::OK,
        "owner get preview meta",
    )
    .await;
}

#[tokio::test]
async fn edited_thumbs_are_scoped_to_the_owner() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_asset_for_admin(&server, id).await;
    mock_original(&server, id).await;
    let (admin, member) = two_owner_apps(&server).await;
    let hash = save_admin_edits(&admin, id).await;

    expect_status(
        &member,
        get(&format!("/api/assets/{id}/edited-thumb?h={hash}&size=128")),
        StatusCode::NOT_FOUND,
        "member edited thumb",
    )
    .await;
    expect_status(
        &admin,
        get(&format!("/api/assets/{id}/edited-thumb?h={hash}&size=128")),
        StatusCode::OK,
        "owner edited thumb",
    )
    .await;
}

async fn mock_original(server: &MockServer, id: uuid::Uuid) {
    let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../raw-pipeline/tests/fixtures/Sony_ILCE-7S_14bit_14bit_compressed_3-2.arw");
    let bytes = std::fs::read(&file).expect("committed Sony ARW fixture");
    Mock::given(method("GET"))
        .and(path(format!("/api/assets/{id}/original")))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "image/x-sony-arw")
                .set_body_bytes(bytes),
        )
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/api/assets/{id}/original")))
        .and(header("x-api-key", MEMBER_API_KEY))
        .respond_with(ResponseTemplate::new(403))
        .mount(server)
        .await;
}
