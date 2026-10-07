use crate::common::*;
use axum::http::StatusCode;
use little_exif::exif_tag::ExifTag;
use little_exif::ifd::ExifTagGroup;
use raw_pipeline::frame::RawFrame;
use serde_json::{Value, json};
use tower::ServiceExt;
use wiremock::{MockServer, ResponseTemplate};

async fn export(server: &MockServer, body: Value) -> axum::response::Response {
    test_app(server)
        .await
        .oneshot(json_request(
            "POST",
            &format!("/api/assets/{}/export", asset_id()),
            body,
        ))
        .await
        .unwrap()
}

async fn export_frame(body: Value) -> RawFrame {
    let server = MockServer::start().await;
    mock_original(&server, asset_id()).await;
    mock_asset_detail(&server).await;
    let resp = export(&server, body.clone()).await;
    if resp.status() != StatusCode::OK {
        panic!("status {} for {body}", resp.status());
    }
    raw_pipeline::decode::decode(&body_bytes(resp).await).unwrap()
}

async fn export_dims(body: Value) -> (usize, usize) {
    let frame = export_frame(body).await;
    (frame.meta.width, frame.meta.height)
}

fn neighbour_contrast(frame: &RawFrame) -> f64 {
    frame
        .data
        .iter()
        .zip(&frame.data[frame.cpp..])
        .map(|(a, b)| f64::from((b - a).abs()))
        .sum()
}

fn jpeg_with_camera_and_gps() -> Vec<u8> {
    let mut meta = little_exif::metadata::Metadata::new();
    meta.set_tag(ExifTag::Make("SONY".into()));
    meta.set_tag(ExifTag::GPSLatitudeRef("N".into()));
    meta.set_tag(ExifTag::GPSLatitude(vec![59u32.into(); 3]));
    let format = raw_pipeline::frame::OutputFormat::Jpeg {
        quality: 90,
        subsampling: raw_pipeline::frame::JpegSubsampling::Chroma420,
    };
    let source = raw_pipeline::metadata::ExportMetadata {
        exif: Some(meta),
        location: true,
    };
    let (embedded, _) = raw_pipeline::encode::embedded(
        Some(&source),
        &format,
        raw_pipeline::frame::OutputColorSpace::SRgb,
        (64, 48),
    );
    raw_pipeline::encode::encode_from_rgb8(
        &vec![128u8; 64 * 48 * 3],
        64,
        48,
        &format,
        raw_pipeline::frame::OutputColorSpace::SRgb,
        embedded.as_ref(),
    )
    .unwrap()
}

#[tokio::test]
async fn export_returns_full_res_jpeg() {
    let server = MockServer::start().await;
    mock_original(&server, asset_id()).await;
    mock_asset_detail(&server).await;

    let resp = export(
        &server,
        json!({"edits": {}, "filename_template": "{date}_{name}"}),
    )
    .await;
    if resp.status() != StatusCode::OK {
        panic!("status {}", resp.status());
    }
    let disp = header_str(&resp, "content-disposition").unwrap_or_default();
    if disp
        != "attachment; filename=\"2026-01-01_DSC0001.jpg\"; filename*=UTF-8''2026-01-01_DSC0001.jpg"
    {
        panic!("disposition: {disp}");
    }
    let bytes = body_bytes(resp).await;
    if &bytes[..2] != b"\xff\xd8" {
        panic!("not jpeg");
    }
    if bytes.len() < 100_000 {
        panic!("full res suspiciously small: {} bytes", bytes.len());
    }
}

#[tokio::test]
async fn export_output_sharpening_raises_local_contrast() {
    let plain = export_frame(json!({
        "edits": {},
        "format": "png",
        "resize_mode": "dimensions",
        "resize_width": 600,
        "resize_height": 600
    }))
    .await;
    let sharpened = export_frame(json!({
        "edits": {},
        "format": "png",
        "resize_mode": "dimensions",
        "resize_width": 600,
        "resize_height": 600,
        "output_sharpen_media": "matte",
        "output_sharpen_amount": "high"
    }))
    .await;
    if (plain.meta.width, plain.meta.height) != (sharpened.meta.width, sharpened.meta.height) {
        panic!("output sharpening changed the size");
    }
    let before = neighbour_contrast(&plain);
    let after = neighbour_contrast(&sharpened);
    if after <= before * 1.05 {
        panic!("sharpened contrast {after} vs plain {before}");
    }
}

#[tokio::test]
async fn export_rejects_invalid_options() {
    for body in [
        json!({
            "edits": {},
            "output_sharpen_media": "glossy",
            "output_sharpen_ppi": 20
        }),
        json!({ "edits": {}, "resize_mode": "dimensions" }),
    ] {
        let server = MockServer::start().await;
        let resp = export(&server, body.clone()).await;
        if resp.status() != StatusCode::BAD_REQUEST {
            panic!("status {} for {body}", resp.status());
        }
    }
}

#[tokio::test]
async fn export_fits_the_requested_dimensions() {
    let boxed = export_dims(json!({
        "edits": {},
        "resize_mode": "dimensions",
        "resize_width": 800,
        "resize_height": 800
    }))
    .await;
    if boxed.0.max(boxed.1) != 800 {
        panic!("800x800 export is {boxed:?}");
    }
    let wide = export_dims(json!({
        "edits": {},
        "resize_mode": "dimensions",
        "resize_width": 400
    }))
    .await;
    if wide.0 != 400 {
        panic!("400 wide export is {wide:?}");
    }
}

#[tokio::test]
async fn export_enlarges_a_small_crop_only_when_asked() {
    let edits = json!({
        "geometry": { "crop": { "x": 0.4, "y": 0.4, "w": 0.1, "h": 0.1 } }
    });
    let kept = export_dims(json!({
        "edits": edits,
        "resize_mode": "dimensions",
        "resize_width": 1500,
        "resize_height": 1500
    }))
    .await;
    if kept.0.max(kept.1) >= 1500 {
        panic!("crop was enlarged without asking: {kept:?}");
    }
    let enlarged = export_dims(json!({
        "edits": edits,
        "resize_mode": "dimensions",
        "resize_width": 1500,
        "resize_height": 1500,
        "resize_enlarge": true
    }))
    .await;
    if enlarged.0.max(enlarged.1) != 1500 {
        panic!("enlarged crop is {enlarged:?}");
    }
}

#[tokio::test]
async fn export_metadata_keeps_or_strips_camera_and_location() {
    for (label, mut body, want_make, want_gps) in [
        ("all", json!({ "metadata": "all" }), true, true),
        (
            "no-location",
            json!({ "metadata": "no-location" }),
            true,
            false,
        ),
        ("none", json!({ "metadata": "none" }), false, false),
        ("legacy off", json!({ "include_exif": false }), false, false),
    ] {
        let server = MockServer::start().await;
        mock_original_with(
            &server,
            asset_id(),
            ResponseTemplate::new(200)
                .insert_header("content-type", "image/jpeg")
                .set_body_bytes(jpeg_with_camera_and_gps()),
        )
        .await;
        mock_asset_detail(&server).await;
        body["edits"] = json!({});
        let resp = export(&server, body).await;
        if resp.status() != StatusCode::OK {
            panic!("{label}: status {}", resp.status());
        }
        let bytes = body_bytes(resp).await;
        let parsed = raw_pipeline::metadata::read(&bytes);
        let has_make = parsed.as_ref().is_some_and(|m| {
            m.into_iter()
                .any(|t| matches!(t, ExifTag::Make(v) if v == "SONY"))
        });
        let has_gps = parsed
            .as_ref()
            .is_some_and(|m| m.into_iter().any(|t| t.get_group() == ExifTagGroup::GPS));
        if has_make != want_make || has_gps != want_gps {
            panic!("{label}: make {has_make}, gps {has_gps}");
        }
    }
}

#[tokio::test]
async fn export_download_reports_metadata_warnings() {
    for (label, original, metadata, want) in [
        (
            "no exif",
            plain_jpeg(),
            "all",
            Some(json!([
                "Metadata not copied: no readable EXIF in the original"
            ])),
        ),
        ("no exif, none", plain_jpeg(), "none", None),
        ("exif", jpeg_with_camera_and_gps(), "all", None),
    ] {
        let server = MockServer::start().await;
        mock_original_with(
            &server,
            asset_id(),
            ResponseTemplate::new(200)
                .insert_header("content-type", "image/jpeg")
                .set_body_bytes(original),
        )
        .await;
        mock_asset_detail(&server).await;
        let resp = export(&server, json!({ "edits": {}, "metadata": metadata })).await;
        if resp.status() != StatusCode::OK {
            panic!("{label}: status {}", resp.status());
        }
        let got = resp
            .headers()
            .get("x-export-warnings")
            .map(|v| serde_json::from_slice::<Value>(v.as_bytes()).unwrap());
        if got != want {
            panic!("{label}: warnings {got:?}");
        }
    }
}
