use crate::common::*;
use axum::body::Body;
use axum::http::{HeaderValue, Request, StatusCode};
use raw_pipeline::ai_denoise::DenoisePlane;
use serde_json::{Value, json};
use tower::ServiceExt;
use wiremock::MockServer;

fn ai_denoise_manifest() -> Value {
    json!({"schema_version": 2, "ops": {"ai_denoise": {"strength": 100.0}}})
}

fn with_etag(mut req: Request<Body>, etag: &str) -> Request<Body> {
    req.headers_mut()
        .insert("if-none-match", HeaderValue::from_str(etag).unwrap());
    req
}

struct Fixture {
    state: immich_edit_backend::state::AppState,
    app: axum::Router,
    identity: immich_edit_backend::services::render::RenderIdentity,
}

async fn fixture(server: &MockServer) -> Fixture {
    mock_original(server, asset_id()).await;
    mock_asset_detail(server).await;
    let state = ml_enabled(test_state(server).await);
    install_fake_model(&state).await;
    let token = seed_session(server, &state).await;
    let identity = identity_for(&state, test_user_id()).await;
    let app = wrap_auth(router(state.clone()), token);
    Fixture {
        state,
        app,
        identity,
    }
}

async fn edited_entry(app: &axum::Router) -> (String, String, String) {
    let resp = app.clone().oneshot(get("/api/edits")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let list = body_json(resp).await;
    let item = &list["items"][0];
    (
        list["render_revision"].as_str().unwrap().to_string(),
        item["hash"].as_str().unwrap().to_string(),
        item["revision"].as_str().unwrap().to_string(),
    )
}

async fn thumb(
    app: &axum::Router,
    hash: &str,
    revision: &str,
    etag: Option<&str>,
) -> axum::response::Response {
    let uri = format!(
        "/api/assets/{}/edited-thumb?h={hash}&r={revision}&size=128",
        asset_id()
    );
    let req = match etag {
        Some(etag) => with_etag(get(&uri), etag),
        None => get(&uri),
    };
    app.clone().oneshot(req).await.unwrap()
}

#[tokio::test]
async fn a_bake_moves_the_edited_thumb_to_a_new_revision() {
    let server = MockServer::start().await;
    let f = fixture(&server).await;
    let resp = f
        .app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("/api/assets/{}/edits", asset_id()),
            ai_denoise_manifest(),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let (base, hash, pending) = edited_entry(&f.app).await;
    assert_eq!(pending, format!("{base}-dn"));
    let resp = thumb(&f.app, &hash, &pending, None).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        header_str(&resp, "cache-control").as_deref(),
        Some("private, max-age=0, must-revalidate"),
        "a thumb without its plane must not be pinned"
    );
    let pending_etag = header_str(&resp, "etag").unwrap();
    let resp = thumb(&f.app, &hash, &pending, Some(&pending_etag)).await;
    assert_eq!(resp.status(), StatusCode::NOT_MODIFIED);

    let key = plane_key(f.identity, asset_id());
    let plane_id = key.plane_id();
    put_grey_plane(&f.state, key).await;

    let (_, _, ready) = edited_entry(&f.app).await;
    assert_eq!(ready, format!("{base}-dn{plane_id:x}"));
    let resp = thumb(&f.app, &hash, &pending, Some(&pending_etag)).await;
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "a finished bake must invalidate the cached thumb"
    );
    let ready_etag = header_str(&resp, "etag").unwrap();
    assert_ne!(ready_etag, pending_etag);
    assert_eq!(
        header_str(&resp, "cache-control").as_deref(),
        Some("private, max-age=0, must-revalidate"),
        "a stale r must not pin the new image"
    );

    let resp = thumb(&f.app, &hash, &ready, None).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        header_str(&resp, "cache-control").as_deref(),
        Some("private, max-age=31536000, immutable")
    );
    assert_eq!(header_str(&resp, "etag").unwrap(), ready_etag);
}

#[tokio::test]
async fn a_plane_that_does_not_fit_is_dropped_and_never_pinned() {
    let server = MockServer::start().await;
    let f = fixture(&server).await;
    let id = asset_id();
    let resp = f
        .app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("/api/assets/{id}/edits"),
            ai_denoise_manifest(),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let key = plane_key(f.identity, id);
    let misfit = DenoisePlane::new(key.plane_id(), 8, 8, vec![0; 8 * 8 * 3]).unwrap();
    let store = f.state.render.denoise().store();
    let ticket = store.ticket(key.owner).await;
    assert!(
        store
            .put(key.clone(), misfit, ticket)
            .await
            .unwrap()
            .is_some()
    );

    let (base, hash, ready) = edited_entry(&f.app).await;
    let resp = thumb(&f.app, &hash, &ready, None).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        header_str(&resp, "cache-control").as_deref(),
        Some("private, max-age=0, must-revalidate"),
        "a thumb rendered without its plane must not be pinned"
    );
    let without_plane = body_bytes(resp).await;
    let (_, _, revision) = edited_entry(&f.app).await;
    assert_eq!(revision, format!("{base}-dn"));
    let resp = f
        .app
        .clone()
        .oneshot(get(&format!("/api/assets/{id}/preview?max=256")))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    put_grey_plane(&f.state, key).await;
    let resp = thumb(&f.app, &hash, &ready, None).await;
    assert_eq!(
        header_str(&resp, "cache-control").as_deref(),
        Some("private, max-age=31536000, immutable")
    );
    assert_ne!(
        body_bytes(resp).await,
        without_plane,
        "the thumb rendered without the plane must not be cached under its revision"
    );
}

fn probe(uri: &str, body: &Option<Value>, etag: Option<&str>) -> Request<Body> {
    let req = match body {
        Some(body) => json_request("POST", uri, body.clone()),
        None => get(uri),
    };
    match etag {
        Some(etag) => with_etag(req, etag),
        None => req,
    }
}

#[tokio::test]
async fn a_bake_changes_every_etag_that_shows_the_plane() {
    let server = MockServer::start().await;
    let f = fixture(&server).await;
    let id = asset_id();
    let resp = f
        .app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("/api/assets/{id}/edits"),
            ai_denoise_manifest(),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let resp = f
        .app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/presets",
            json!({"name": "Clean", "manifest": ai_denoise_manifest()}),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let preset = body_json(resp).await["id"].as_str().unwrap().to_string();
    let source_body = json!({"max_edge": 256, "edits": {"detail": {"ai_denoise": true}}});
    let cases = [
        (
            "preview",
            format!("/api/assets/{id}/preview?max=512"),
            None,
            true,
        ),
        (
            "source",
            format!("/api/assets/{id}/source"),
            Some(source_body),
            false,
        ),
        (
            "preset thumb",
            format!("/api/assets/{id}/presets/{preset}/thumb"),
            None,
            false,
        ),
    ];

    let mut pending = Vec::new();
    for (label, uri, body, _) in &cases {
        let resp = f.app.clone().oneshot(probe(uri, body, None)).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "{label}");
        let etag = header_str(&resp, "etag").unwrap();
        let bytes = body_bytes(resp).await;
        let resp = f
            .app
            .clone()
            .oneshot(probe(uri, body, Some(&etag)))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_MODIFIED, "{label}");
        pending.push((etag, bytes));
    }

    put_grey_plane(&f.state, plane_key(f.identity, id)).await;
    assert_eq!(
        denoise_call(&f.app, id, "GET").await,
        json!({"state": "ready"})
    );

    for ((label, uri, body, pixels_change), (etag, bytes)) in cases.iter().zip(pending) {
        let resp = f
            .app
            .clone()
            .oneshot(probe(uri, body, Some(&etag)))
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "a finished bake must invalidate the {label}"
        );
        assert_ne!(header_str(&resp, "etag").unwrap(), etag, "{label}");
        if *pixels_change {
            assert_ne!(
                body_bytes(resp).await,
                bytes,
                "the baked plane must change the {label}"
            );
        }
    }
}
