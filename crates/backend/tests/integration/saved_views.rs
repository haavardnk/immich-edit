use crate::common::*;
use axum::http::StatusCode;
use tower::ServiceExt;
use wiremock::MockServer;

async fn create(app: &axum::Router, name: &str, url: &str) -> serde_json::Value {
    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/saved-views",
            serde_json::json!({ "name": name, "url": url }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    body_json(resp).await
}

async fn list(app: &axum::Router) -> serde_json::Value {
    let resp = app.clone().oneshot(get("/api/saved-views")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    body_json(resp).await
}

#[tokio::test]
async fn a_saved_view_survives_create_list_update_and_delete() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;

    let created = create(&app, "  Five stars  ", "/photos?rating=5").await;
    assert_eq!(created["name"], "Five stars");
    assert_eq!(created["url"], "/photos?rating=5");
    assert_eq!(created["created_at"], created["updated_at"]);
    let id = created["id"].as_str().unwrap().to_string();

    let resp = app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("/api/saved-views/{id}"),
            serde_json::json!({ "name": "Recent picks", "url": "/photos?rating=3&sort=date" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let updated = body_json(resp).await;
    assert_eq!(updated["id"], id.as_str());
    assert_eq!(updated["created_at"], created["created_at"]);

    let listed = list(&app).await;
    assert_eq!(listed.as_array().map(Vec::len), Some(1));
    assert_eq!(listed[0]["name"], "Recent picks");
    assert_eq!(listed[0]["url"], "/photos?rating=3&sort=date");

    let resp = app
        .clone()
        .oneshot(empty_request("DELETE", &format!("/api/saved-views/{id}")))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    assert_eq!(list(&app).await.as_array().map(Vec::len), Some(0));
}

#[tokio::test]
async fn saved_views_list_by_name_ignoring_case() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    for name in ["beta", "Gamma", "alpha", "Beta two"] {
        create(&app, name, "/photos").await;
    }
    let names: Vec<String> = list(&app)
        .await
        .as_array()
        .unwrap()
        .iter()
        .map(|view| view["name"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(names, ["alpha", "beta", "Beta two", "Gamma"]);
}

#[tokio::test]
async fn saved_view_bodies_are_validated() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    for (body, message) in [
        (
            serde_json::json!({ "name": "  ", "url": "/photos" }),
            "saved view name required",
        ),
        (
            serde_json::json!({ "name": "Evil", "url": "//evil.example" }),
            "saved view url must be a path on this site",
        ),
    ] {
        let resp = app
            .clone()
            .oneshot(json_request("POST", "/api/saved-views", body.clone()))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "{body}");
        let json = body_json(resp).await;
        assert_eq!(json["code"], "bad_request", "{body}");
        assert_eq!(json["message"], message, "{body}");
    }
    assert_eq!(list(&app).await.as_array().map(Vec::len), Some(0));
}

#[tokio::test]
async fn an_unknown_field_is_unprocessable() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/saved-views",
            serde_json::json!({ "name": "Picks", "url": "/photos", "icon": "star" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(list(&app).await.as_array().map(Vec::len), Some(0));
}

#[tokio::test]
async fn an_unknown_saved_view_is_not_found() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let id = uuid::Uuid::new_v4();
    let resp = app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("/api/saved-views/{id}"),
            serde_json::json!({ "name": "Picks", "url": "/photos" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let resp = app
        .oneshot(empty_request("DELETE", &format!("/api/saved-views/{id}")))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn an_invalid_body_is_rejected_before_the_lookup() {
    let server = MockServer::start().await;
    let app = test_app(&server).await;
    let resp = app
        .oneshot(json_request(
            "PUT",
            &format!("/api/saved-views/{}", uuid::Uuid::new_v4()),
            serde_json::json!({ "name": "Evil", "url": "//evil.example" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(body_json(resp).await["code"], "bad_request");
}

#[tokio::test]
async fn a_rebind_removes_all_saved_views() {
    let server = MockServer::start().await;
    let (state, admin, member) = two_owner_state(&server).await;
    create(&admin, "Admin view", "/photos").await;
    create(&member, "Member view", "/albums").await;

    rebind_instance(&server, &state).await;

    for owner in [test_user_id(), member_user_id()] {
        assert!(
            state
                .edits
                .list_saved_views(owner)
                .await
                .unwrap()
                .is_empty()
        );
    }
}
