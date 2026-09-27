use axum::body::Body;
use axum::http::Request;
use axum::response::Response;
use http_body_util::BodyExt;
use serde_json::Value;

pub async fn body_bytes(resp: Response) -> Vec<u8> {
    resp.into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec()
}

pub async fn body_json(resp: Response) -> Value {
    serde_json::from_slice(&body_bytes(resp).await).unwrap()
}

pub fn header_str(resp: &Response, name: &str) -> Option<String> {
    resp.headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
}

pub fn flow_cookie(resp: &Response) -> String {
    resp.headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find(|v| v.starts_with("immich_edit_oauth=") && !v.starts_with("immich_edit_oauth=;"))
        .map(|v| v.split(';').next().unwrap().to_string())
        .expect("no oauth flow cookie")
}

pub fn get(uri: &str) -> Request<Body> {
    Request::builder().uri(uri).body(Body::empty()).unwrap()
}

pub fn empty_request(method: &str, uri: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

pub fn json_request(method: &str, uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}
