use axum::body::Body;
use axum::http::{Request, StatusCode};
use immich_edit_backend::asset_key::AssetKey;
use immich_edit_backend::routes::export::{ExportToImmichBody, hash_request};
use immich_edit_backend::services::edits_store::{
    ExportJobBaseline, ExportJobKey, ExportOutputKey,
};
use immich_edit_backend::state::AppState;
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::matchers::{body_json as body_is, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::{
    asset_id, mock_named_asset, mock_trash_feature, named_asset_detail, seed_and_wrap, test_state,
    test_user_id,
};

pub const EDIT_NAME: &str = "DSC0001_edit.jpg";

pub enum PreviousExport {
    Live,
    Trashed,
    Foreign,
    Answers(u16),
}

pub struct Conflicts {
    pub server: MockServer,
    pub state: AppState,
    pub app: axum::Router,
}

pub fn master() -> AssetKey {
    AssetKey::master(asset_id())
}

pub fn output_key(asset: AssetKey, preset: &str) -> ExportOutputKey<'_> {
    ExportOutputKey {
        owner: test_user_id(),
        asset_id: asset,
        preset,
    }
}

pub fn export_job(asset: AssetKey, key: &str) -> ExportJobKey<'_> {
    ExportJobKey {
        owner: test_user_id(),
        asset_id: asset,
        key,
    }
}

pub fn conflict_body(on_conflict: &str) -> Value {
    json!({ "metadata": "none", "on_conflict": on_conflict })
}

pub fn baseline(previous: Uuid, replace: bool) -> ExportJobBaseline {
    ExportJobBaseline {
        previous_asset_id: Some(previous),
        replace_asset_id: replace.then_some(previous),
    }
}

impl Conflicts {
    pub async fn start(trash: bool) -> Self {
        let server = MockServer::start().await;
        mock_named_asset(&server, asset_id(), "DSC0001.ARW").await;
        mock_trash_feature(&server, trash).await;
        let state = test_state(&server).await;
        let app = seed_and_wrap(&server, state.clone()).await;
        Self { server, state, app }
    }

    pub async fn previous(&self, preset: &str, answer: PreviousExport) -> Uuid {
        let id = Uuid::new_v4();
        self.track(master(), preset, id).await;
        self.answer(id, answer).await;
        id
    }

    pub async fn track(&self, asset: AssetKey, preset: &str, id: Uuid) {
        let recorded = self
            .state
            .edits
            .put_export_output(output_key(asset, preset).output(id, None))
            .await
            .unwrap();
        assert!(recorded, "{asset} {preset} already tracked");
    }

    pub async fn answer(&self, id: Uuid, answer: PreviousExport) {
        let mut detail = named_asset_detail(id, EDIT_NAME);
        let response = match answer {
            PreviousExport::Live => ResponseTemplate::new(200).set_body_json(detail),
            PreviousExport::Trashed => {
                detail["isTrashed"] = json!(true);
                ResponseTemplate::new(200).set_body_json(detail)
            }
            PreviousExport::Foreign => {
                detail["ownerId"] = json!(Uuid::new_v4());
                ResponseTemplate::new(200).set_body_json(detail)
            }
            PreviousExport::Answers(status) => ResponseTemplate::new(status)
                .set_body_json(json!({ "message": "Not found or no asset.read access" })),
        };
        Mock::given(method("GET"))
            .and(path(format!("/api/assets/{id}")))
            .respond_with(response)
            .mount(&self.server)
            .await;
    }

    pub async fn uploads_as(&self, id: Uuid, status: &str) {
        Mock::given(method("POST"))
            .and(path("/api/assets"))
            .respond_with(
                ResponseTemplate::new(201).set_body_json(json!({ "id": id, "status": status })),
            )
            .up_to_n_times(1)
            .mount(&self.server)
            .await;
        Mock::given(method("GET"))
            .and(path(format!("/api/assets/{id}")))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(named_asset_detail(id, EDIT_NAME)),
            )
            .mount(&self.server)
            .await;
    }

    pub async fn expect_replace(&self, target: Uuid, upload: Uuid, times: u64) {
        Mock::given(method("PUT"))
            .and(path("/api/assets/copy"))
            .and(body_is(json!({
                "sourceId": target,
                "targetId": upload,
                "sidecar": false,
                "albums": true,
                "sharedLinks": true,
                "favorite": true,
                "stack": true,
            })))
            .respond_with(ResponseTemplate::new(204))
            .expect(times)
            .mount(&self.server)
            .await;
        Mock::given(method("DELETE"))
            .and(path("/api/assets"))
            .and(body_is(json!({ "ids": [target], "force": false })))
            .respond_with(ResponseTemplate::new(204))
            .expect(times)
            .mount(&self.server)
            .await;
    }

    pub async fn accept_replaces(&self) {
        for (verb, url) in [("PUT", "/api/assets/copy"), ("DELETE", "/api/assets")] {
            Mock::given(method(verb))
                .and(path(url))
                .respond_with(ResponseTemplate::new(204))
                .mount(&self.server)
                .await;
        }
    }

    pub async fn seed_uploaded(
        &self,
        key: &str,
        request: &Value,
        upload: Uuid,
        baseline: ExportJobBaseline,
    ) {
        let body: ExportToImmichBody = serde_json::from_value(request.clone()).unwrap();
        let job = export_job(master(), key);
        let hash = hash_request(master(), &body);
        let edits = &self.state.edits;
        assert!(
            edits
                .reserve_export_job(job, &hash, baseline)
                .await
                .unwrap()
        );
        edits
            .put_export_job_uploaded(job, &hash, upload, EDIT_NAME, "created", &[])
            .await
            .unwrap();
    }

    pub async fn export(&self, key: Option<&str>, body: &Value) -> (StatusCode, Value) {
        self.export_asset(master(), key, body).await
    }

    pub async fn export_asset(
        &self,
        asset: AssetKey,
        key: Option<&str>,
        body: &Value,
    ) -> (StatusCode, Value) {
        let mut request = Request::builder()
            .method("POST")
            .uri(format!("/api/assets/{asset}/export/immich"))
            .header("content-type", "application/json");
        if let Some(key) = key {
            request = request.header("idempotency-key", key);
        }
        let resp = self
            .app
            .clone()
            .oneshot(request.body(Body::from(body.to_string())).unwrap())
            .await
            .unwrap();
        let status = resp.status();
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        )
    }

    pub async fn recorded(&self, preset: &str) -> Option<Uuid> {
        self.recorded_for(master(), preset).await
    }

    pub async fn recorded_for(&self, asset: AssetKey, preset: &str) -> Option<Uuid> {
        self.state
            .edits
            .get_export_output(output_key(asset, preset))
            .await
            .unwrap()
    }

    pub async fn calls(&self, verb: &str, url: &str) -> usize {
        self.server
            .received_requests()
            .await
            .unwrap()
            .iter()
            .filter(|r| r.method.as_str() == verb && r.url.path() == url)
            .count()
    }

    pub async fn uploads(&self) -> usize {
        self.calls("POST", "/api/assets").await
    }

    pub async fn trashed(&self) -> Vec<Value> {
        self.server
            .received_requests()
            .await
            .unwrap()
            .iter()
            .filter(|r| r.method.as_str() == "DELETE" && r.url.path() == "/api/assets")
            .map(|r| serde_json::from_slice::<Value>(&r.body).unwrap()["ids"].clone())
            .collect()
    }
}

pub fn warnings(json: &Value) -> Vec<&str> {
    json["warnings"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect()
}
