use std::sync::Arc;
use std::time::Duration;

use axum::http::StatusCode;
use immich_edit_backend::config::{Config, MlRuntimeMode};
use immich_edit_backend::services::denoise::DenoiseService;
use immich_edit_backend::services::denoise_store::PlaneKey;
use immich_edit_backend::services::ml_runtime::InferencePermits;
use immich_edit_backend::services::model_download::Downloaded;
use immich_edit_backend::services::render::RenderIdentity;
use immich_edit_backend::state::AppState;
use raw_pipeline::ai_denoise::DenoisePlane;
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;

use super::{ARW_FIXTURE, body_json, empty_request, raw_fixture};

pub const DENOISE_MODEL: &str = "nind_utnet";

pub fn with_denoise(
    mut state: AppState,
    tweak: impl FnOnce(&mut Config),
) -> (AppState, InferencePermits) {
    let mut config = (*state.config).clone();
    config.ml_runtime = MlRuntimeMode::Cpu;
    tweak(&mut config);
    let permits = InferencePermits::new(config.ml_max_concurrency);
    state.denoise = DenoiseService::new(
        &config,
        state.render.clone(),
        state.models.clone(),
        permits.clone(),
    );
    state.config = Arc::new(config);
    (state, permits)
}

pub fn ml_enabled(state: AppState) -> AppState {
    with_denoise(state, |_| {}).0
}

async fn install_denoise_model(state: &AppState, name: &str, bytes: &[u8]) {
    let entry = ml::catalog::find(DENOISE_MODEL).unwrap();
    let path = state.models.dir().join(name);
    std::fs::create_dir_all(state.models.dir()).unwrap();
    std::fs::write(&path, bytes).unwrap();
    let downloaded = Downloaded {
        path,
        hash: entry.sha256.to_string(),
        len: bytes.len() as u64,
    };
    state
        .models
        .install_downloaded(entry, downloaded, None)
        .await
        .unwrap();
}

pub async fn install_fake_model(state: &AppState) {
    install_denoise_model(state, "fake-nind.onnx", b"not an onnx graph").await;
}

pub async fn install_identity_model(state: &AppState) {
    install_denoise_model(state, "identity-nind.onnx", &identity_onnx()).await;
}

fn varint(mut value: u64, out: &mut Vec<u8>) {
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            out.push(byte);
            return;
        }
        out.push(byte | 0x80);
    }
}

fn int_field(field: u64, value: u64) -> Vec<u8> {
    let mut out = Vec::new();
    varint(field << 3, &mut out);
    varint(value, &mut out);
    out
}

fn bytes_field(field: u64, bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    varint((field << 3) | 2, &mut out);
    varint(bytes.len() as u64, &mut out);
    out.extend_from_slice(bytes);
    out
}

fn float_tensor_value(name: &str) -> Vec<u8> {
    let dims: Vec<u8> = ["n", "c", "h", "w"]
        .iter()
        .flat_map(|dim| bytes_field(1, &bytes_field(2, dim.as_bytes())))
        .collect();
    let tensor = [int_field(1, 1), bytes_field(2, &dims)].concat();
    let value_type = bytes_field(1, &tensor);
    [bytes_field(1, name.as_bytes()), bytes_field(2, &value_type)].concat()
}

pub fn identity_onnx() -> Vec<u8> {
    let node = [
        bytes_field(1, b"x"),
        bytes_field(2, b"y"),
        bytes_field(4, b"Identity"),
    ]
    .concat();
    let graph = [
        bytes_field(1, &node),
        bytes_field(2, b"identity"),
        bytes_field(11, &float_tensor_value("x")),
        bytes_field(12, &float_tensor_value("y")),
    ]
    .concat();
    let opset = int_field(2, 13);
    [
        int_field(1, 7),
        bytes_field(7, &graph),
        bytes_field(8, &opset),
    ]
    .concat()
}

pub async fn identity_for(state: &AppState, immich_id: Uuid) -> RenderIdentity {
    let owner = state
        .auth
        .get_user(immich_id)
        .await
        .unwrap()
        .expect("seed the session first")
        .id;
    RenderIdentity {
        owner,
        server_epoch: state.instance.get().await.unwrap().server_epoch,
    }
}

pub fn plane_key(identity: RenderIdentity, source: Uuid) -> PlaneKey {
    PlaneKey::new(identity, source, DENOISE_MODEL.into())
}

pub async fn put_grey_plane(state: &AppState, key: PlaneKey) {
    let frame = raw_pipeline::decode::decode(&raw_fixture(ARW_FIXTURE)).unwrap();
    let grey = 0x2e66;
    let plane = DenoisePlane::new(
        key.plane_id(),
        frame.meta.width,
        frame.meta.height,
        vec![grey; frame.meta.width * frame.meta.height * 3],
    )
    .unwrap();
    let store = state.render.denoise().store();
    let ticket = store.ticket(key.owner).await;
    assert!(store.put(key, plane, ticket).await.unwrap().is_some());
}

pub async fn denoise_call(app: &axum::Router, id: Uuid, method: &str) -> Value {
    let uri = format!("/api/assets/{id}/denoise");
    let resp = app
        .clone()
        .oneshot(empty_request(method, &uri))
        .await
        .unwrap();
    if resp.status() != StatusCode::OK {
        panic!("{method} {uri}: {}", resp.status());
    }
    body_json(resp).await
}

pub async fn settle_denoise(app: &axum::Router, id: Uuid) -> Value {
    for _ in 0..1200 {
        let status = denoise_call(app, id, "GET").await;
        if !matches!(status["state"].as_str(), Some("queued" | "running")) {
            return status;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("denoise never settled");
}

pub async fn wait_for_denoise_state(app: &axum::Router, id: Uuid, want: &str) {
    for _ in 0..1200 {
        if denoise_call(app, id, "GET").await["state"] == want {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("denoise never reached {want}");
}
