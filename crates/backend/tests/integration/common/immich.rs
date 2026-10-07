use uuid::Uuid;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::{MEMBER_API_KEY, TEST_API_KEY};

pub fn album_id() -> Uuid {
    Uuid::parse_str("11111111-2222-3333-4444-555555555555").unwrap()
}

pub fn asset_id() -> Uuid {
    Uuid::parse_str("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee").unwrap()
}

pub fn arw_response() -> ResponseTemplate {
    raw_response(
        "Sony_ILCE-7S_14bit_14bit_compressed_3-2.arw",
        "image/x-sony-arw",
    )
}

pub fn raw_response(fixture: &str, content_type: &str) -> ResponseTemplate {
    let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../raw-pipeline/tests/fixtures")
        .join(fixture);
    ResponseTemplate::new(200)
        .insert_header("content-type", content_type)
        .set_body_bytes(std::fs::read(file).expect("committed RAW fixture"))
}

pub fn plain_jpeg() -> Vec<u8> {
    let rgb = vec![128u8; 64 * 48 * 3];
    raw_pipeline::encode::encode_jpeg_rgb(
        raw_pipeline::encode::ImageRgb8 {
            rgb: &rgb,
            width: 64,
            height: 48,
        },
        90,
        raw_pipeline::frame::JpegSubsampling::Chroma420,
        raw_pipeline::frame::OutputColorSpace::SRgb,
        None,
    )
    .unwrap()
}

pub async fn mock_original_with(server: &MockServer, id: Uuid, response: ResponseTemplate) {
    Mock::given(method("GET"))
        .and(path(format!("/api/assets/{id}/original")))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(response)
        .mount(server)
        .await;
}

pub async fn mock_original(server: &MockServer, id: Uuid) {
    mock_original_with(server, id, arw_response()).await;
}

pub async fn originals_fetched(server: &MockServer, id: Uuid) -> usize {
    let wanted = format!("/api/assets/{id}/original");
    server
        .received_requests()
        .await
        .unwrap_or_default()
        .iter()
        .filter(|r| r.url.path() == wanted)
        .count()
}

pub async fn mock_original_owned_by_admin(server: &MockServer, id: Uuid) {
    mock_original(server, id).await;
    Mock::given(method("GET"))
        .and(path(format!("/api/assets/{id}/original")))
        .and(header("x-api-key", MEMBER_API_KEY))
        .respond_with(ResponseTemplate::new(403))
        .mount(server)
        .await;
}

pub async fn mock_albums(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/api/albums"))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "id": album_id(),
                "albumName": "Test Album",
                "assetCount": 3,
                "updatedAt": "2026-01-01T00:00:00Z"
            }
        ])))
        .mount(server)
        .await;
}

pub async fn mock_album_detail(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path(format!("/api/albums/{}", album_id())))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": album_id(),
            "albumName": "Test Album",
            "assetCount": 1,
            "updatedAt": "2026-01-01T00:00:00Z"
        })))
        .mount(server)
        .await;
}

pub async fn mock_search_metadata(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/api/search/metadata"))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "assets": {
                "items": [{
                    "id": asset_id(),
                    "originalFileName": "DSC0001.ARW",
                    "type": "IMAGE"
                }],
                "count": 1,
                "total": 1,
                "nextPage": null
            }
        })))
        .mount(server)
        .await;
}

pub async fn mock_thumb(server: &MockServer, size: &str) {
    Mock::given(method("GET"))
        .and(path(format!("/api/assets/{}/thumbnail", asset_id())))
        .and(query_param("size", size))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "image/jpeg")
                .set_body_bytes(vec![0xFFu8, 0xD8, 0xFF, 0xE0, 0x00, 0x10]),
        )
        .mount(server)
        .await;
}

pub async fn mock_ping_ok(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/api/server/ping"))
        .respond_with(ResponseTemplate::new(200).set_body_string("pong"))
        .mount(server)
        .await;
}

pub async fn mock_ping_status(server: &MockServer, status: u16) {
    Mock::given(method("GET"))
        .and(path("/api/server/ping"))
        .respond_with(ResponseTemplate::new(status))
        .mount(server)
        .await;
}

pub async fn mock_asset_detail(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path(format!("/api/assets/{}", asset_id())))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": asset_id(),
            "originalFileName": "DSC0001.ARW",
            "type": "IMAGE",
            "originalMimeType": "image/x-sony-arw",
            "fileCreatedAt": "2026-01-01T00:00:00Z",
            "updatedAt": "2026-01-02T00:00:00Z",
            "checksum": "abc",
            "isFavorite": true,
            "exifInfo": {
                "make": "SONY",
                "model": "ILCE-7M4",
                "lensModel": "FE 35mm F1.8",
                "fNumber": 2.8,
                "focalLength": 35.0,
                "iso": 400,
                "exposureTime": "0.004",
                "exifImageWidth": 4032,
                "exifImageHeight": 3024,
                "dateTimeOriginal": "2026-01-01T00:00:00Z",
                "timeZone": "Europe/Oslo",
                "latitude": 59.91,
                "longitude": 10.75,
                "description": "Harbour at dawn",
                "rating": 4,
                "fileSizeInByte": 12345678u64
            },
            "tags": [
                { "id": "11111111-aaaa-bbbb-cccc-000000000001", "name": "Landscape", "value": "Landscape" }
            ]
        })))
        .mount(server)
        .await;
}

pub async fn mock_asset_update(server: &MockServer) {
    Mock::given(method("PATCH"))
        .and(path(format!("/api/assets/{}", asset_id())))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": asset_id(),
            "originalFileName": "DSC0001.ARW",
            "type": "IMAGE",
            "isFavorite": true,
            "exifInfo": { "rating": 5 },
            "tags": []
        })))
        .mount(server)
        .await;
}

pub async fn mock_tag_upsert(server: &MockServer) {
    Mock::given(method("PUT"))
        .and(path("/api/tags"))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "id": "22222222-aaaa-bbbb-cccc-000000000002", "name": "New", "value": "New" }
        ])))
        .mount(server)
        .await;
}

pub fn tag_id() -> Uuid {
    Uuid::parse_str("33333333-aaaa-bbbb-cccc-000000000003").unwrap()
}

pub async fn mock_tag_asset(server: &MockServer) {
    Mock::given(method("PUT"))
        .and(path(format!("/api/tags/{}/assets", tag_id())))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "id": asset_id(), "success": true }
        ])))
        .mount(server)
        .await;
}

pub async fn mock_untag_asset(server: &MockServer) {
    Mock::given(method("DELETE"))
        .and(path(format!("/api/tags/{}/assets", tag_id())))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "id": asset_id(), "success": true }
        ])))
        .mount(server)
        .await;
}

pub async fn mock_tag_list_with_stats(server: &MockServer, count: u64) {
    Mock::given(method("GET"))
        .and(path("/api/tags"))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "id": tag_id(), "name": "Blue", "value": "Blue" }
        ])))
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/search/statistics"))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "total": count
        })))
        .mount(server)
        .await;
}

pub fn person_id() -> Uuid {
    Uuid::parse_str("44444444-aaaa-bbbb-cccc-000000000004").unwrap()
}

pub async fn mock_people_list_with_stats(server: &MockServer, count: u64) {
    Mock::given(method("GET"))
        .and(path("/api/people"))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "people": [ { "id": person_id(), "name": "Alice" } ],
            "total": 1,
            "hasNextPage": false
        })))
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/search/statistics"))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "total": count
        })))
        .mount(server)
        .await;
}
