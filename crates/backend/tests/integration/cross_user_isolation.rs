use crate::common::*;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

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

async fn save_admin_edits(admin: &axum::Router, id: uuid::Uuid) -> String {
    let saved = body_json(
        expect_status(
            admin,
            json_request(
                "PUT",
                &format!("/api/assets/{id}/edits"),
                exposure_manifest(1.5),
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
        body_json(expect_status(&admin, get("/api/edits"), StatusCode::OK, "owner list").await)
            .await;
    if owner_list["items"].as_array().map(Vec::len) != Some(1) {
        panic!("owner edit list: {owner_list}");
    }

    let record = body_json(
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

    let history = body_json(
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
        body_json(expect_status(&member, get("/api/edits"), StatusCode::OK, "member list").await)
            .await;
    if member_list["items"].as_array().map(Vec::len) != Some(0) {
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
        json_request(
            "PUT",
            &format!("/api/assets/{id}/edits"),
            exposure_manifest(-2.0),
        ),
        StatusCode::BAD_GATEWAY,
        "member put edits",
    )
    .await;
    expect_status(
        &member,
        empty_request("DELETE", &format!("/api/assets/{id}/edits")),
        StatusCode::NO_CONTENT,
        "member delete edits",
    )
    .await;

    let record = body_json(
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
async fn history_restore_is_scoped_to_the_owner() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_asset_for_admin(&server, id).await;
    let (admin, member) = two_owner_apps(&server).await;
    save_admin_edits(&admin, id).await;
    expect_status(
        &admin,
        json_request(
            "PUT",
            &format!("/api/assets/{id}/edits"),
            exposure_manifest(0.5),
        ),
        StatusCode::OK,
        "owner second put",
    )
    .await;
    let history_uri = format!("/api/assets/{id}/edits/history");
    let history =
        body_json(expect_status(&admin, get(&history_uri), StatusCode::OK, "owner history").await)
            .await;
    let older = history[1]["id"].as_i64().expect("older entry id");
    let restore = || {
        json_request(
            "POST",
            &format!("/api/assets/{id}/edits/restore"),
            serde_json::json!({ "entry_id": older }),
        )
    };

    expect_status(
        &member,
        restore(),
        StatusCode::NOT_FOUND,
        "member restore owner entry",
    )
    .await;
    let history =
        body_json(expect_status(&admin, get(&history_uri), StatusCode::OK, "owner history").await)
            .await;
    if history
        .as_array()
        .is_none_or(|h| h.iter().any(|e| e["undone"] != false))
    {
        panic!("member restore touched the owner's history: {history}");
    }

    let restored =
        body_json(expect_status(&admin, restore(), StatusCode::OK, "owner restore").await).await;
    if restored["manifest"]["ops"]["exposure"]["ev"] != 1.5 {
        panic!("owner restore body: {restored}");
    }
}

#[tokio::test]
async fn virtual_copies_are_scoped_to_the_owner() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_asset_for_admin(&server, id).await;
    let (admin, member) = two_owner_apps(&server).await;

    let copy = body_json(
        expect_status(
            &admin,
            json_request(
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

    let listed = body_json(
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
        json_request(
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
        empty_request("DELETE", &format!("/api/copies/{copy_id}")),
        StatusCode::NOT_FOUND,
        "member delete copy",
    )
    .await;

    let still_there = body_json(
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

    let preset = body_json(
        expect_status(
            &admin,
            json_request(
                "POST",
                "/api/presets",
                serde_json::json!({ "name": "Owner look", "manifest": exposure_manifest(0.5) }),
            ),
            StatusCode::OK,
            "owner create preset",
        )
        .await,
    )
    .await;
    let preset_id = preset["id"].as_str().expect("preset id").to_string();

    let listed =
        body_json(expect_status(&member, get("/api/presets"), StatusCode::OK, "member list").await)
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
        json_request(
            "PUT",
            &format!("/api/presets/{preset_id}"),
            serde_json::json!({ "name": "stolen", "manifest": exposure_manifest(9.0) }),
        ),
        StatusCode::NOT_FOUND,
        "member update preset",
    )
    .await;
    expect_status(
        &member,
        json_request(
            "PUT",
            &format!("/api/presets/{preset_id}/favorite"),
            serde_json::json!({ "favorite": true }),
        ),
        StatusCode::NOT_FOUND,
        "member star preset",
    )
    .await;
    expect_status(
        &member,
        empty_request("DELETE", &format!("/api/presets/{preset_id}")),
        StatusCode::NOT_FOUND,
        "member delete preset",
    )
    .await;

    expect_status(
        &member,
        get(&format!(
            "/api/assets/{}/presets/{preset_id}/thumb",
            asset_id()
        )),
        StatusCode::NOT_FOUND,
        "member preset thumb",
    )
    .await;

    let owned = body_json(
        expect_status(
            &admin,
            get(&format!("/api/presets/{preset_id}")),
            StatusCode::OK,
            "owner get preset",
        )
        .await,
    )
    .await;
    if owned["name"] != "Owner look" || owned["favorite"] != false {
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
        body_json(expect_status(&admin, upload, StatusCode::OK, "owner upload raster").await).await;
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
        body_json(expect_status(&admin, upload, StatusCode::OK, "owner upload raster").await).await;
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

    let job = body_json(
        expect_status(
            &admin,
            json_request(
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

    let listed = body_json(
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
        empty_request("POST", &format!("/api/jobs/{job_id}/cancel")),
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
    mock_original_owned_by_admin(&server, id).await;
    let (admin, member) = two_owner_apps(&server).await;

    let resp = expect_status(
        &admin,
        json_request(
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
    mock_original_owned_by_admin(&server, id).await;
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

#[tokio::test]
async fn export_presets_are_scoped_to_the_owner() {
    let server = MockServer::start().await;
    let (admin, member) = two_owner_apps(&server).await;

    let preset = body_json(
        expect_status(
            &admin,
            json_request(
                "POST",
                "/api/export-presets",
                serde_json::json!({ "name": "Web JPEG", "form": { "format": "jpeg" } }),
            ),
            StatusCode::OK,
            "owner create export preset",
        )
        .await,
    )
    .await;
    let preset_id = preset["id"].as_str().expect("export preset id").to_string();

    let listed = body_json(
        expect_status(
            &member,
            get("/api/export-presets"),
            StatusCode::OK,
            "member list",
        )
        .await,
    )
    .await;
    if listed.as_array().map(Vec::len) != Some(0) {
        panic!("member listed the owner's export presets: {listed}");
    }
    expect_status(
        &member,
        json_request(
            "PUT",
            &format!("/api/export-presets/{preset_id}"),
            serde_json::json!({ "name": "stolen", "form": {} }),
        ),
        StatusCode::NOT_FOUND,
        "member update export preset",
    )
    .await;
    expect_status(
        &member,
        empty_request("DELETE", &format!("/api/export-presets/{preset_id}")),
        StatusCode::NOT_FOUND,
        "member delete export preset",
    )
    .await;

    let owned = body_json(
        expect_status(
            &admin,
            get("/api/export-presets"),
            StatusCode::OK,
            "owner list",
        )
        .await,
    )
    .await;
    if owned[0]["name"] != "Web JPEG" || owned[0]["form"]["format"] != "jpeg" {
        panic!("owner export preset after member calls: {owned}");
    }
}

#[tokio::test]
async fn appearance_is_scoped_to_the_owner() {
    let server = MockServer::start().await;
    let (admin, member) = two_owner_apps(&server).await;
    let chosen = serde_json::json!({
        "mode": "light",
        "light_theme": "paper",
        "dark_theme": "midnight",
        "accents": { "blue": "sky" }
    });

    expect_status(
        &admin,
        json_request("PUT", "/api/appearance", chosen.clone()),
        StatusCode::OK,
        "owner put appearance",
    )
    .await;

    let seen = body_json(
        expect_status(
            &member,
            get("/api/appearance"),
            StatusCode::OK,
            "member get appearance",
        )
        .await,
    )
    .await;
    if seen["mode"] != "dark"
        || seen["light_theme"] != "immich-edit-light"
        || seen["dark_theme"] != "immich-edit-dark"
        || seen["accents"] != serde_json::json!({})
    {
        panic!("member read the owner's appearance: {seen}");
    }

    expect_status(
        &member,
        json_request(
            "PUT",
            "/api/appearance",
            serde_json::json!({
                "mode": "system",
                "light_theme": "stolen",
                "dark_theme": "stolen",
                "accents": {}
            }),
        ),
        StatusCode::OK,
        "member put appearance",
    )
    .await;

    let owned = body_json(
        expect_status(
            &admin,
            get("/api/appearance"),
            StatusCode::OK,
            "owner get appearance",
        )
        .await,
    )
    .await;
    if owned != chosen {
        panic!("owner appearance after member calls: {owned}");
    }
}

#[tokio::test]
async fn preferences_are_scoped_to_the_owner() {
    let server = MockServer::start().await;
    let (admin, member) = two_owner_apps(&server).await;
    let chosen = body_json(
        expect_status(
            &admin,
            json_request(
                "PATCH",
                "/api/preferences",
                serde_json::json!({
                    "click_target": "loupe",
                    "mask_overlay_color": "#ff0000",
                    "shortcuts": { "help": ["F1"] },
                    "crop_ratios": [{ "num": 3, "den": 2 }]
                }),
            ),
            StatusCode::OK,
            "owner patch preferences",
        )
        .await,
    )
    .await;

    let seen = body_json(
        expect_status(
            &member,
            get("/api/preferences"),
            StatusCode::OK,
            "member get preferences",
        )
        .await,
    )
    .await;
    if seen["click_target"] != "editor"
        || !seen["mask_overlay_color"].is_null()
        || seen["shortcuts"] != serde_json::json!({})
        || !seen["crop_ratios"].is_null()
    {
        panic!("member read the owner's preferences: {seen}");
    }

    let member_saved = body_json(
        expect_status(
            &member,
            json_request(
                "PATCH",
                "/api/preferences",
                serde_json::json!({
                    "pinned_settings": ["exposure"],
                    "crop_ratios": [{ "num": 7, "den": 5 }]
                }),
            ),
            StatusCode::OK,
            "member patch preferences",
        )
        .await,
    )
    .await;
    if member_saved["click_target"] != "editor"
        || member_saved["shortcuts"] != serde_json::json!({})
        || member_saved["crop_ratios"] != serde_json::json!([{ "num": 7, "den": 5 }])
    {
        panic!("member patch merged onto the owner's preferences: {member_saved}");
    }

    let owned = body_json(
        expect_status(
            &admin,
            get("/api/preferences"),
            StatusCode::OK,
            "owner get preferences",
        )
        .await,
    )
    .await;
    if owned != chosen
        || !owned["pinned_settings"].is_null()
        || owned["crop_ratios"] != serde_json::json!([{ "num": 3, "den": 2 }])
    {
        panic!("owner preferences after member calls: {owned}");
    }
}

#[tokio::test]
async fn saved_views_are_scoped_to_the_owner() {
    let server = MockServer::start().await;
    let (admin, member) = two_owner_apps(&server).await;

    let view = body_json(
        expect_status(
            &admin,
            json_request(
                "POST",
                "/api/saved-views",
                serde_json::json!({ "name": "Five stars", "url": "/photos?rating=5" }),
            ),
            StatusCode::OK,
            "owner create saved view",
        )
        .await,
    )
    .await;
    let view_id = view["id"].as_str().expect("saved view id").to_string();

    let listed = body_json(
        expect_status(
            &member,
            get("/api/saved-views"),
            StatusCode::OK,
            "member list",
        )
        .await,
    )
    .await;
    if listed.as_array().map(Vec::len) != Some(0) {
        panic!("member listed the owner's saved views: {listed}");
    }
    expect_status(
        &member,
        json_request(
            "PUT",
            &format!("/api/saved-views/{view_id}"),
            serde_json::json!({ "name": "stolen", "url": "/stolen" }),
        ),
        StatusCode::NOT_FOUND,
        "member update saved view",
    )
    .await;
    expect_status(
        &member,
        empty_request("DELETE", &format!("/api/saved-views/{view_id}")),
        StatusCode::NOT_FOUND,
        "member delete saved view",
    )
    .await;

    let owned = body_json(
        expect_status(
            &admin,
            get("/api/saved-views"),
            StatusCode::OK,
            "owner list",
        )
        .await,
    )
    .await;
    if owned.as_array().map(Vec::len) != Some(1) || owned[0] != view {
        panic!("owner saved views after member calls: {owned}");
    }
}

#[tokio::test]
async fn snapshots_are_scoped_to_the_owner() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_asset_for_admin(&server, id).await;
    let (admin, member) = two_owner_apps(&server).await;
    let snapshots_uri = format!("/api/assets/{id}/snapshots");

    let snapshot = body_json(
        expect_status(
            &admin,
            json_request(
                "POST",
                &snapshots_uri,
                serde_json::json!({ "name": "Owner look", "manifest": exposure_manifest(1.5) }),
            ),
            StatusCode::OK,
            "owner create snapshot",
        )
        .await,
    )
    .await;
    let snapshot_uri = format!("{snapshots_uri}/{}", snapshot["id"].as_str().unwrap());

    let listed = body_json(
        expect_status(
            &member,
            get(&snapshots_uri),
            StatusCode::OK,
            "member list snapshots",
        )
        .await,
    )
    .await;
    if listed.as_array().map(Vec::len) != Some(0) {
        panic!("member listed the owner's snapshots: {listed}");
    }
    expect_status(
        &member,
        json_request(
            "PATCH",
            &snapshot_uri,
            serde_json::json!({ "name": "stolen" }),
        ),
        StatusCode::NOT_FOUND,
        "member rename snapshot",
    )
    .await;
    expect_status(
        &member,
        json_request(
            "PATCH",
            &snapshot_uri,
            serde_json::json!({ "manifest": exposure_manifest(4.0) }),
        ),
        StatusCode::NOT_FOUND,
        "member update snapshot",
    )
    .await;
    expect_status(
        &member,
        empty_request("POST", &format!("{snapshot_uri}/restore")),
        StatusCode::NOT_FOUND,
        "member restore snapshot",
    )
    .await;
    expect_status(
        &member,
        empty_request("DELETE", &snapshot_uri),
        StatusCode::NOT_FOUND,
        "member delete snapshot",
    )
    .await;

    let owned = body_json(
        expect_status(
            &admin,
            get(&snapshots_uri),
            StatusCode::OK,
            "owner list snapshots",
        )
        .await,
    )
    .await;
    if owned.as_array().map(Vec::len) != Some(1) || owned[0] != snapshot {
        panic!("owner snapshots after member calls: {owned}");
    }
    let restored = body_json(
        expect_status(
            &admin,
            empty_request("POST", &format!("{snapshot_uri}/restore")),
            StatusCode::OK,
            "owner restore snapshot",
        )
        .await,
    )
    .await;
    if restored["manifest"]["ops"]["exposure"]["ev"] != 1.5 {
        panic!("owner restore body: {restored}");
    }
}

#[cfg(feature = "ml")]
async fn source_etag(app: &axum::Router, id: uuid::Uuid, what: &str) -> String {
    let resp = expect_status(
        app,
        json_request(
            "POST",
            &format!("/api/assets/{id}/source"),
            serde_json::json!({"max_edge": 256, "edits": {"detail": {"ai_denoise": true}}}),
        ),
        StatusCode::OK,
        what,
    )
    .await;
    header_str(&resp, "etag").expect("source etag")
}

#[cfg(feature = "ml")]
#[tokio::test]
async fn denoise_planes_are_scoped_to_the_owner() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_original(&server, id).await;
    Mock::given(method("GET"))
        .and(path(format!("/api/assets/{id}/original")))
        .and(header("x-api-key", MEMBER_API_KEY))
        .respond_with(arw_response())
        .mount(&server)
        .await;
    let state = ml_enabled(test_state(&server).await);
    install_fake_model(&state).await;
    let (admin, member) = two_owner_routers(&server, state.clone()).await;
    let owner = identity_for(&state, test_user_id()).await;
    let key = plane_key(owner, id);
    let owner_plane = format!("dn{:x}", key.plane_id());
    put_grey_plane(&state, key).await;

    assert_eq!(denoise_call(&admin, id, "GET").await["state"], "ready");
    assert_eq!(denoise_call(&member, id, "GET").await["state"], "idle");
    let started = denoise_call(&member, id, "POST").await;
    assert!(
        matches!(started["state"].as_str(), Some("queued" | "running")),
        "the member must bake their own plane, not reuse the owner's: {started}"
    );
    assert_eq!(settle_denoise(&member, id).await["state"], "failed");

    let owner_etag = source_etag(&admin, id, "owner source").await;
    assert!(owner_etag.contains(&owner_plane), "{owner_etag}");
    let member_etag = source_etag(&member, id, "member source").await;
    assert!(
        !member_etag.contains(&owner_plane),
        "the member's source must not use the owner's plane: {member_etag}"
    );
    assert_eq!(denoise_call(&admin, id, "GET").await["state"], "ready");
}
