mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::*;
use http_body_util::BodyExt;
use raw_pipeline::source::{self, LinearKind};
use tower::ServiceExt;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const CUBE: &str = "LUT_3D_SIZE 2\n0 0 0\n1 0 0\n0 1 0\n1 1 0\n0 0 1\n1 0 1\n0 1 1\n1 1 1\n";

async fn body_bytes(resp: axum::response::Response) -> Vec<u8> {
    resp.into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec()
}

fn header_str(resp: &axum::response::Response, name: &str) -> Option<String> {
    resp.headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
}

async fn mock_arw_original(server: &MockServer, id: uuid::Uuid) {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../raw-pipeline/tests/fixtures/Sony_ILCE-7S_14bit_14bit_compressed_3-2.arw");
    Mock::given(method("GET"))
        .and(path(format!("/api/assets/{id}/original")))
        .and(header("x-api-key", "test-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "image/x-sony-arw")
                .set_body_bytes(std::fs::read(fixture).expect("committed Sony ARW fixture")),
        )
        .mount(server)
        .await;
}

fn post_source(id: uuid::Uuid, body: serde_json::Value, etag: Option<&str>) -> Request<Body> {
    let mut req = Request::builder()
        .method("POST")
        .uri(format!("/api/assets/{id}/source"))
        .header("content-type", "application/json")
        .header("accept-encoding", "gzip");
    if let Some(etag) = etag {
        req = req.header("if-none-match", etag);
    }
    req.body(Body::from(body.to_string())).unwrap()
}

fn post_bytes(uri: &str, bytes: Vec<u8>) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .body(Body::from(bytes))
        .unwrap()
}

#[tokio::test]
async fn source_returns_a_decodable_linear_source() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_arw_original(&server, id).await;
    let app = test_app(&server).await;

    let body = serde_json::json!({"max_edge": 512, "edits": {"basic": {"exposure_ev": 1.0}}});
    let resp = app.oneshot(post_source(id, body, None)).await.unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    if header_str(&resp, "content-type").as_deref() != Some("application/vnd.immich-edit.source") {
        panic!("content-type: {:?}", header_str(&resp, "content-type"));
    }
    if resp.headers().contains_key("content-encoding") {
        panic!("an already compressed source was compressed again");
    }
    if header_str(&resp, "etag").is_none() {
        panic!("missing etag");
    }
    let image = source::decode(&body_bytes(resp).await).unwrap();
    let (w, h) = image.header.dims;
    if w.max(h) > 512 || w.max(h) < 256 {
        panic!("source dims {w}x{h} do not follow max_edge 512");
    }
    if image.header.kind != LinearKind::PostWb || image.header.atmosphere.is_none() {
        panic!("source is not portable: {:?}", image.header.kind);
    }
    if !image.header.meta.is_raw {
        panic!("raw metadata lost");
    }
}

#[tokio::test]
async fn a_source_roi_windows_the_region() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_arw_original(&server, id).await;
    let app = test_app(&server).await;

    let whole = app
        .clone()
        .oneshot(post_source(id, serde_json::json!({"max_edge": 256}), None))
        .await
        .unwrap();
    let whole_etag = header_str(&whole, "etag").expect("etag");
    let whole = source::decode(&body_bytes(whole).await).unwrap();
    let tile = app
        .oneshot(post_source(
            id,
            serde_json::json!({"max_edge": 256, "roi": [0.375, 0.375, 0.25, 0.25]}),
            Some(&whole_etag),
        ))
        .await
        .unwrap();
    if tile.status() != StatusCode::OK || header_str(&tile, "etag").as_deref() == Some(&whole_etag)
    {
        panic!("a roi must be its own source: {}", tile.status());
    }
    let tile = source::decode(&body_bytes(tile).await).unwrap();
    let Some(window) = tile.header.window else {
        panic!("a quarter of the frame was not windowed");
    };
    let (w, h) = tile.header.dims;
    if window.full.0 <= whole.header.dims.0 * 2 || w >= window.full.0 || h >= window.full.1 {
        panic!(
            "tile {w}x{h} in {:?} is not a window at tile resolution (whole {:?})",
            window.full, whole.header.dims
        );
    }
}

#[tokio::test]
async fn source_etag_ignores_display_edits() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_arw_original(&server, id).await;
    let app = test_app(&server).await;

    let first = app
        .clone()
        .oneshot(post_source(
            id,
            serde_json::json!({"max_edge": 512, "edits": {"basic": {"exposure_ev": 1.0}}}),
            None,
        ))
        .await
        .unwrap();
    let etag = header_str(&first, "etag").expect("etag");

    let display_only = app
        .clone()
        .oneshot(post_source(
            id,
            serde_json::json!({"max_edge": 512, "edits": {"basic": {"exposure_ev": -2.0, "wb_temp": 30.0}}}),
            Some(&etag),
        ))
        .await
        .unwrap();
    if display_only.status() != StatusCode::NOT_MODIFIED {
        panic!("a display edit must revalidate: {}", display_only.status());
    }

    let noise = app
        .oneshot(post_source(
            id,
            serde_json::json!({"max_edge": 512, "edits": {"detail": {"luma_nr_amount": 40.0}}}),
            Some(&etag),
        ))
        .await
        .unwrap();
    if noise.status() != StatusCode::OK {
        panic!(
            "a noise reduction change must re-render: {}",
            noise.status()
        );
    }
    if header_str(&noise, "etag").as_deref() == Some(etag.as_str()) {
        panic!("noise reduction did not change the etag");
    }
}

#[tokio::test]
async fn source_names_the_dcp_it_was_built_with() {
    let server = MockServer::start().await;
    let id = asset_id();
    mock_arw_original(&server, id).await;
    let app = test_app(&server).await;

    let dcp_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/dcp/Canon EOS 5D.dcp");
    let dcp_bytes = std::fs::read(dcp_path).unwrap();
    let imported = app
        .clone()
        .oneshot(post_bytes(
            "/api/dcp?name=probe&camera=ILCE-7S",
            dcp_bytes.clone(),
        ))
        .await
        .unwrap();
    if imported.status() != StatusCode::CREATED {
        panic!("dcp import: {}", imported.status());
    }
    let meta: serde_json::Value = serde_json::from_slice(&body_bytes(imported).await).unwrap();
    let dcp_id = meta["id"].as_str().unwrap().to_string();

    let body = serde_json::json!({
        "max_edge": 512,
        "edits": {"color": {"dcp": {"mode": "profile", "profile_id": dcp_id}}}
    });
    let resp = app
        .clone()
        .oneshot(post_source(id, body, None))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    if header_str(&resp, "x-source-dcp").as_deref() != Some(dcp_id.as_str()) {
        panic!("x-source-dcp: {:?}", header_str(&resp, "x-source-dcp"));
    }

    let raw = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/dcp/{dcp_id}/raw"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    if raw.status() != StatusCode::OK {
        panic!("dcp raw status {}", raw.status());
    }
    if !header_str(&raw, "cache-control").is_some_and(|v| v.contains("immutable")) {
        panic!(
            "dcp raw cache-control: {:?}",
            header_str(&raw, "cache-control")
        );
    }
    if body_bytes(raw).await != dcp_bytes {
        panic!("dcp raw bytes differ from the import");
    }
}

#[tokio::test]
async fn lut_cube_is_served_as_imported() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let imported = app
        .clone()
        .oneshot(post_bytes("/api/luts?name=probe", CUBE.as_bytes().to_vec()))
        .await
        .unwrap();
    if imported.status() != StatusCode::CREATED {
        panic!("lut import: {}", imported.status());
    }
    let meta: serde_json::Value = serde_json::from_slice(&body_bytes(imported).await).unwrap();
    let lut_id = meta["id"].as_str().unwrap().to_string();

    let get = |uri: String| {
        let app = app.clone();
        async move {
            app.oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
                .await
                .unwrap()
        }
    };
    let cube = get(format!("/api/luts/{lut_id}/cube")).await;
    if cube.status() != StatusCode::OK {
        panic!("cube status {}", cube.status());
    }
    if !header_str(&cube, "cache-control").is_some_and(|v| v.contains("immutable")) {
        panic!(
            "cube cache-control: {:?}",
            header_str(&cube, "cache-control")
        );
    }
    if body_bytes(cube).await != CUBE.as_bytes() {
        panic!("cube bytes differ from the import");
    }
    for uri in [
        "/api/luts/missing/cube".to_string(),
        "/api/dcp/missing/raw".to_string(),
    ] {
        let resp = get(uri.clone()).await;
        if resp.status() != StatusCode::NOT_FOUND {
            panic!("{uri}: {}", resp.status());
        }
    }
}
