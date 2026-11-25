use crate::common::search_library::{Photo, mount};
use crate::common::*;
use axum::http::StatusCode;
use serde_json::{Value, json};
use wiremock::matchers::{body_json as body_is, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn mock_map_styles(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/api/server/config"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "oauthButtonText": "",
            "mapLightStyleUrl": "https://tiles.immich.cloud/v1/style/light.json",
            "mapDarkStyleUrl": "https://tiles.immich.cloud/v1/style/dark.json"
        })))
        .mount(server)
        .await;
}

async fn put_settings(app: &axum::Router, body: Value) -> (StatusCode, Value) {
    send_json(app, "PUT", "/api/admin/map-settings", Some(body)).await
}

#[tokio::test]
async fn the_map_follows_immich_styles_until_an_admin_sets_custom_ones() {
    let server = MockServer::start().await;
    mock_map_styles(&server).await;
    mock_me(&server, test_user_id(), true).await;
    let app = test_app(&server).await;

    let (status, config) = send_json(&app, "GET", "/api/map/config", None).await;
    if status != StatusCode::OK
        || config["source"] != "immich"
        || config["dark"] != "https://tiles.immich.cloud/v1/style/dark.json"
        || config["search"] != true
    {
        panic!("{status}: {config}");
    }

    let custom = json!({
        "styleLight": "https://tiles.openfreemap.org/styles/liberty",
        "styleDark": "https://tiles.openfreemap.org/styles/dark",
        "geocoderUrl": ""
    });
    let (status, saved) = put_settings(&app, custom.clone()).await;
    if status != StatusCode::OK || saved != custom {
        panic!("{status}: {saved}");
    }
    let (_, config) = send_json(&app, "GET", "/api/map/config", None).await;
    if config["source"] != "custom"
        || config["light"] != "https://tiles.openfreemap.org/styles/liberty"
        || config["search"] != false
    {
        panic!("custom: {config}");
    }
    let (_, read) = send_json(&app, "GET", "/api/admin/map-settings", None).await;
    if read != custom {
        panic!("read back: {read}");
    }
}

#[tokio::test]
async fn map_settings_reject_half_or_bad_urls_and_non_admins() {
    let server = MockServer::start().await;
    mock_me(&server, test_user_id(), true).await;
    let app = test_app(&server).await;
    for body in [
        json!({ "styleLight": "https://a.example/l.json", "styleDark": null, "geocoderUrl": "" }),
        json!({ "styleLight": null, "styleDark": null, "geocoderUrl": "photon" }),
        json!({ "styleLight": null, "styleDark": null, "geocoderUrl": "", "key": "x" }),
    ] {
        let (status, json) = put_settings(&app, body.clone()).await;
        if !status.is_client_error() {
            panic!("{body}: {status} {json}");
        }
    }
    let member = member_app(&server).await;
    let (status, _) = send_json(&member, "GET", "/api/admin/map-settings", None).await;
    if status != StatusCode::FORBIDDEN {
        panic!("member read {status}");
    }
}

#[tokio::test]
async fn markers_forward_the_favorite_filter_and_drop_videos() {
    let server = MockServer::start().await;
    let video = uuid::Uuid::from_u128(0xdead_beef);
    Mock::given(method("GET"))
        .and(path("/api/map/markers"))
        .and(query_param("isFavorite", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "id": asset_id(), "lat": 60.3975, "lon": 5.324, "city": "Bergen", "state": "Vestland", "country": "Norway" },
            { "id": video, "lat": 60.4, "lon": 5.3, "city": "Bergen", "state": "Vestland", "country": "Norway" }
        ])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/search/metadata"))
        .and(body_is(json!({
            "type": "VIDEO",
            "size": 1000,
            "page": 1,
            "withExif": false,
            "isFavorite": true
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "assets": {
                "items": [{ "id": video, "originalFileName": "clip.mp4", "type": "VIDEO" }],
                "count": 1,
                "total": 1,
                "nextPage": null
            }
        })))
        .expect(1)
        .mount(&server)
        .await;
    let app = test_app(&server).await;
    let (status, markers) = send_json(&app, "GET", "/api/map/markers?favorite=true", None).await;
    if status != StatusCode::OK
        || markers.as_array().map(Vec::len) != Some(1)
        || markers[0]["city"] != "Bergen"
        || markers[0]["lat"] != 60.3975
    {
        panic!("{status}: {markers}");
    }
    for bad in [
        "/api/map/markers?from=yesterday",
        "/api/map/markers?archived=true",
    ] {
        let (status, _) = send_json(&app, "GET", bad, None).await;
        if !status.is_client_error() {
            panic!("{bad}: {status}");
        }
    }
}

fn shot(taken: &str, zone_hours: i64) -> Photo {
    Photo {
        id: uuid::Uuid::new_v4(),
        taken: chrono::DateTime::parse_from_rfc3339(taken)
            .unwrap()
            .with_timezone(&chrono::Utc),
        zone: chrono::TimeDelta::hours(zone_hours),
        ..Photo::default()
    }
}

#[tokio::test]
async fn marker_dates_follow_each_photo_clock() {
    let early = shot("2026-10-01T22:30:00Z", 3);
    let noon = shot("2026-10-02T10:00:00Z", 0);
    let late = shot("2026-10-02T22:30:00Z", -3);
    let next = shot("2026-10-02T21:00:00Z", 5);
    let before = shot("2026-10-01T20:00:00Z", 2);
    let photos = [early, noon, late, next, before];
    for minor in [1, 3] {
        let server = MockServer::start().await;
        mock_server_version(&server, 3, minor, 2).await;
        mount(&server, TEST_API_KEY, photos.to_vec()).await;
        let markers: Vec<Value> = [early, noon, late, next]
            .iter()
            .map(|photo| json!({ "id": photo.id, "lat": 60.0, "lon": 5.0 }))
            .collect();
        Mock::given(method("GET"))
            .and(path("/api/map/markers"))
            .and(query_param("fileCreatedAfter", "2026-10-01T22:30:00+00:00"))
            .and(query_param(
                "fileCreatedBefore",
                "2026-10-02T22:30:00+00:00",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!(markers)))
            .expect(1)
            .mount(&server)
            .await;
        let app = test_app(&server).await;
        let (status, found) = send_json(
            &app,
            "GET",
            "/api/map/markers?from=2026-10-02T00:00:00&until=2026-10-03T00:00:00",
            None,
        )
        .await;
        let ids: Vec<&str> = found
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item["id"].as_str())
                    .collect()
            })
            .unwrap_or_default();
        let wanted = [early.id, noon.id, late.id].map(|id| id.to_string());
        if status != StatusCode::OK
            || ids.len() != 3
            || !wanted.iter().all(|id| ids.contains(&id.as_str()))
        {
            panic!("3.{minor}: {status} {found}");
        }
    }
}

#[tokio::test]
async fn reverse_geocode_names_the_nearest_place_or_nothing() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/map/reverse-geocode"))
        .and(query_param("lat", "60.3975"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "city": "Bergen", "state": "Vestland", "country": "Norway" }
        ])))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/map/reverse-geocode"))
        .and(query_param("lat", "0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&server)
        .await;
    let app = test_app(&server).await;
    let (_, place) = send_json(
        &app,
        "GET",
        "/api/map/reverse-geocode?lat=60.3975&lon=5.324",
        None,
    )
    .await;
    if place["city"] != "Bergen" {
        panic!("{place}");
    }
    let (status, none) =
        send_json(&app, "GET", "/api/map/reverse-geocode?lat=0&lon=-30", None).await;
    if status != StatusCode::OK || !none.is_null() {
        panic!("{status}: {none}");
    }
    let (status, _) = send_json(&app, "GET", "/api/map/reverse-geocode?lat=91&lon=0", None).await;
    if status != StatusCode::BAD_REQUEST {
        panic!("out of range: {status}");
    }
}

#[tokio::test]
async fn place_search_proxies_photon_once_per_query_and_can_be_off() {
    let photon = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api"))
        .and(query_param("q", "Bryggen"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "type": "FeatureCollection",
            "features": [{
                "geometry": { "coordinates": [5.324, 60.3975] },
                "properties": { "name": "Bryggen", "city": "Bergen", "country": "Norway" }
            }]
        })))
        .expect(1)
        .mount(&photon)
        .await;
    let server = MockServer::start().await;
    mock_me(&server, test_user_id(), true).await;
    let app = test_app(&server).await;
    let geocoder = json!({ "styleLight": null, "styleDark": null, "geocoderUrl": photon.uri() });
    put_settings(&app, geocoder).await;

    for query in ["Bryggen", "%20Bryggen%20"] {
        let (status, places) =
            send_json(&app, "GET", &format!("/api/map/search?q={query}"), None).await;
        if status != StatusCode::OK
            || places
                != json!([{ "name": "Bryggen", "detail": "Bergen, Norway", "lat": 60.3975, "lon": 5.324 }])
        {
            panic!("{query}: {status} {places}");
        }
    }

    put_settings(
        &app,
        json!({ "styleLight": null, "styleDark": null, "geocoderUrl": "" }),
    )
    .await;
    let (status, json) = send_json(&app, "GET", "/api/map/search?q=Bryggen", None).await;
    if status != StatusCode::CONFLICT || json["message"] != "place search is off" {
        panic!("off: {status} {json}");
    }
}

#[tokio::test]
async fn a_location_goes_to_every_source_photo_in_one_update() {
    let server = MockServer::start().await;
    let asset = asset_id();
    Mock::given(method("PUT"))
        .and(path("/api/assets"))
        .and(body_is(
            json!({ "ids": [asset], "latitude": 60.3975, "longitude": 5.324 }),
        ))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    let app = test_app(&server).await;
    let body =
        json!({ "ids": [asset, format!("{asset}_1")], "latitude": 60.3975, "longitude": 5.324 });
    let (status, json) = send_json(&app, "PUT", "/api/assets/location", Some(body)).await;
    if status != StatusCode::NO_CONTENT {
        panic!("{status}: {json}");
    }
}

#[tokio::test]
async fn a_location_is_checked_before_immich() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .respond_with(ResponseTemplate::new(204))
        .expect(0)
        .mount(&server)
        .await;
    let app = test_app(&server).await;
    let asset = asset_id();
    for body in [
        json!({ "ids": [], "latitude": 1.0, "longitude": 1.0 }),
        json!({ "ids": [asset], "latitude": 90.5, "longitude": 1.0 }),
        json!({ "ids": [asset], "latitude": 1.0 }),
        json!({ "ids": [asset], "latitude": 1.0, "longitude": 1.0, "city": "Oslo" }),
        json!({ "ids": vec![asset; 1001], "latitude": 1.0, "longitude": 1.0 }),
    ] {
        let (status, _) = send_json(&app, "PUT", "/api/assets/location", Some(body.clone())).await;
        if !status.is_client_error() {
            panic!("{body}: {status}");
        }
    }
}

#[tokio::test]
async fn per_photo_locations_group_photos_that_share_a_point() {
    let server = MockServer::start().await;
    let (a, b, c) = (
        uuid::Uuid::from_u128(0xb1),
        uuid::Uuid::from_u128(0xb2),
        uuid::Uuid::from_u128(0xb3),
    );
    Mock::given(method("PUT"))
        .and(path("/api/assets"))
        .and(body_is(
            json!({ "ids": [a, c], "latitude": 60.39, "longitude": 5.32 }),
        ))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/assets"))
        .and(body_is(
            json!({ "ids": [b], "latitude": 60.63, "longitude": 6.41 }),
        ))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    let app = test_app(&server).await;
    let body = json!({ "items": [
        { "id": a, "latitude": 60.39, "longitude": 5.32 },
        { "id": b, "latitude": 60.63, "longitude": 6.41 },
        { "id": format!("{a}_1"), "latitude": 1.0, "longitude": 1.0 },
        { "id": c, "latitude": 60.39, "longitude": 5.32 }
    ] });
    let (status, json) = send_json(&app, "PUT", "/api/assets/locations", Some(body)).await;
    assert_eq!(status, StatusCode::OK, "{json}");
    assert_eq!(json, json!({ "failed": [] }));
}

#[tokio::test]
async fn per_photo_locations_report_the_photos_immich_refused() {
    let server = MockServer::start().await;
    let (mine, theirs) = (uuid::Uuid::from_u128(0xc1), uuid::Uuid::from_u128(0xc2));
    Mock::given(method("PUT"))
        .and(path("/api/assets"))
        .and(body_is(
            json!({ "ids": [theirs], "latitude": 2.0, "longitude": 2.0 }),
        ))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({ "message": "no access" })))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/assets"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;
    let app = test_app(&server).await;
    let body = json!({ "items": [
        { "id": mine, "latitude": 1.0, "longitude": 1.0 },
        { "id": theirs, "latitude": 2.0, "longitude": 2.0 }
    ] });
    let (status, json) = send_json(&app, "PUT", "/api/assets/locations", Some(body)).await;
    assert_eq!(status, StatusCode::OK, "{json}");
    assert_eq!(json, json!({ "failed": [theirs] }));

    let only = json!({ "items": [{ "id": theirs, "latitude": 2.0, "longitude": 2.0 }] });
    let (status, _) = send_json(&app, "PUT", "/api/assets/locations", Some(only)).await;
    assert!(status.is_client_error(), "{status}");
}

#[tokio::test]
async fn per_photo_locations_are_checked_before_immich() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .respond_with(ResponseTemplate::new(204))
        .expect(0)
        .mount(&server)
        .await;
    let app = test_app(&server).await;
    let asset = asset_id();
    let item = json!({ "id": asset, "latitude": 1.0, "longitude": 1.0 });
    for body in [
        json!({ "items": [] }),
        json!({ "items": [item, { "id": asset, "latitude": 91.0, "longitude": 1.0 }] }),
        json!({ "items": [{ "id": asset, "latitude": 1.0 }] }),
        json!({ "items": vec![item; 1001] }),
        json!({ "ids": [asset] }),
    ] {
        let (status, _) = send_json(&app, "PUT", "/api/assets/locations", Some(body.clone())).await;
        if !status.is_client_error() {
            panic!("{body}: {status}");
        }
    }
}

#[tokio::test]
async fn summaries_keep_the_order_and_skip_photos_gone_from_immich() {
    let server = MockServer::start().await;
    let first = uuid::Uuid::from_u128(0xa1);
    let gone = uuid::Uuid::from_u128(0xa2);
    let last = uuid::Uuid::from_u128(0xa3);
    let video = uuid::Uuid::from_u128(0xa4);
    for (id, name, kind) in [
        (first, "A.ARW", "IMAGE"),
        (last, "C.ARW", "IMAGE"),
        (video, "V.MP4", "VIDEO"),
    ] {
        Mock::given(method("GET"))
            .and(path(format!("/api/assets/{id}")))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": id, "originalFileName": name, "type": kind
            })))
            .mount(&server)
            .await;
    }
    Mock::given(method("GET"))
        .and(path(format!("/api/assets/{gone}")))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;
    let app = test_app(&server).await;
    let (status, assets) = send_json(
        &app,
        "POST",
        "/api/assets/summaries",
        Some(json!({ "ids": [last, gone, video, first] })),
    )
    .await;
    let names: Vec<&str> = assets
        .as_array()
        .map(|list| {
            list.iter()
                .filter_map(|a| a["originalFileName"].as_str())
                .collect()
        })
        .unwrap_or_default();
    if status != StatusCode::OK || names != ["C.ARW", "A.ARW"] {
        panic!("{status}: {assets}");
    }
}
