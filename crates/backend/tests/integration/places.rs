use crate::common::*;
use axum::http::StatusCode;
use tower::ServiceExt;
use wiremock::MockServer;

async fn send(
    app: &axum::Router,
    method: &str,
    uri: &str,
    body: serde_json::Value,
) -> axum::response::Response {
    app.clone()
        .oneshot(json_request(method, uri, body))
        .await
        .unwrap()
}

async fn places(app: &axum::Router) -> serde_json::Value {
    let resp = app.clone().oneshot(get("/api/places")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    body_json(resp).await
}

fn names(listed: &serde_json::Value) -> Vec<String> {
    listed["saved"]
        .as_array()
        .unwrap()
        .iter()
        .map(|place| place["name"].as_str().unwrap().to_string())
        .collect()
}

fn hytta() -> serde_json::Value {
    serde_json::json!({
        "name": "  Hytta  ",
        "lat": 60.6289,
        "lon": 6.4153,
        "city": "Voss",
        "state": " Vestland ",
        "country": ""
    })
}

#[tokio::test]
async fn a_saved_place_survives_create_list_rename_use_and_delete() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;

    let resp = send(&app, "POST", "/api/places", hytta()).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let created = body_json(resp).await;
    assert_eq!(created["name"], "Hytta");
    assert_eq!(created["lat"], 60.6289);
    assert_eq!(created["city"], "Voss");
    assert_eq!(created["state"], "Vestland");
    assert!(created["country"].is_null());
    let id = created["id"].as_str().unwrap().to_string();

    let resp = send(
        &app,
        "PUT",
        &format!("/api/places/{id}"),
        serde_json::json!({ "name": "Hytta på Voss", "lat": 60.63, "lon": 6.42 }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let updated = body_json(resp).await;
    assert_eq!(updated["name"], "Hytta på Voss");
    assert_eq!(updated["lon"], 6.42);
    assert!(updated["city"].is_null());
    assert_eq!(updated["created_at"], created["created_at"]);

    let resp = send(
        &app,
        "POST",
        "/api/places",
        serde_json::json!({ "name": "Studio", "lat": 59.92, "lon": 10.76 }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(names(&places(&app).await), ["Studio", "Hytta på Voss"]);

    let resp = app
        .clone()
        .oneshot(empty_request("POST", &format!("/api/places/{id}/used")))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    assert_eq!(names(&places(&app).await), ["Hytta på Voss", "Studio"]);

    let resp = app
        .clone()
        .oneshot(empty_request("DELETE", &format!("/api/places/{id}")))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    assert_eq!(names(&places(&app).await), ["Studio"]);
    let resp = app
        .clone()
        .oneshot(empty_request("POST", &format!("/api/places/{id}/used")))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_place_name_is_unique_per_owner_ignoring_case() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    assert_eq!(
        send(&app, "POST", "/api/places", hytta()).await.status(),
        StatusCode::OK
    );

    let resp = send(
        &app,
        "POST",
        "/api/places",
        serde_json::json!({ "name": "HYTTA", "lat": 1.0, "lon": 2.0 }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
    assert_eq!(
        body_json(resp).await["message"],
        "You already have a place called HYTTA"
    );

    let resp = send(
        &app,
        "POST",
        "/api/places",
        serde_json::json!({ "name": "Home", "lat": 1.0, "lon": 2.0 }),
    )
    .await;
    let home = body_json(resp).await["id"].as_str().unwrap().to_string();
    let resp = send(
        &app,
        "PUT",
        &format!("/api/places/{home}"),
        serde_json::json!({ "name": "hytta", "lat": 1.0, "lon": 2.0 }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
    assert_eq!(names(&places(&app).await).len(), 2);
}

#[tokio::test]
async fn a_saved_place_is_validated_before_it_is_stored() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    for body in [
        serde_json::json!({ "name": "  ", "lat": 1.0, "lon": 2.0 }),
        serde_json::json!({ "name": "x".repeat(81), "lat": 1.0, "lon": 2.0 }),
        serde_json::json!({ "name": "Pole", "lat": 90.5, "lon": 2.0 }),
        serde_json::json!({ "name": "Edge", "lat": 1.0, "lon": -180.5 }),
        serde_json::json!({ "name": "Long", "lat": 1.0, "lon": 2.0, "city": "c".repeat(201) }),
        serde_json::json!({ "name": "Ctrl", "lat": 1.0, "lon": 2.0, "country": "No\u{7}rway" }),
    ] {
        let resp = send(&app, "POST", "/api/places", body.clone()).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "{body}");
    }
    let resp = send(
        &app,
        "POST",
        "/api/places",
        serde_json::json!({ "name": "Extra", "lat": 1.0, "lon": 2.0, "zoom": 4 }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert!(places(&app).await["saved"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn recent_places_keep_the_last_eight_and_merge_one_spot() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    for step in 0..10 {
        let body = serde_json::json!({ "lat": 60.0 + f64::from(step), "lon": 5.0, "city": format!("Place {step}") });
        assert_eq!(
            send(&app, "POST", "/api/places/recent", body)
                .await
                .status(),
            StatusCode::OK
        );
    }
    let resp = send(
        &app,
        "POST",
        "/api/places/recent",
        serde_json::json!({ "lat": 65.0002, "lon": 5.0, "city": "Place 5 again" }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let recent = body_json(resp).await;
    let cities: Vec<&str> = recent
        .as_array()
        .unwrap()
        .iter()
        .map(|place| place["city"].as_str().unwrap())
        .collect();
    assert_eq!(
        cities,
        [
            "Place 5 again",
            "Place 9",
            "Place 8",
            "Place 7",
            "Place 6",
            "Place 4",
            "Place 3",
            "Place 2"
        ]
    );
    assert_eq!(places(&app).await["recent"], recent);

    let resp = app
        .clone()
        .oneshot(empty_request("DELETE", "/api/places/recent"))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    assert!(places(&app).await["recent"].as_array().unwrap().is_empty());
}
