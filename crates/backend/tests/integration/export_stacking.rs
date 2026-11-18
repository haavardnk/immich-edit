use crate::common::*;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

const STACK_PATH: &str = r"^/api/stacks/[0-9a-f-]+$";

#[derive(Clone)]
struct FakeStack {
    id: Uuid,
    primary: Uuid,
    members: Vec<Uuid>,
}

impl FakeStack {
    fn to_json(&self) -> Value {
        let assets: Vec<Value> = self.members.iter().map(|id| json!({ "id": id })).collect();
        json!({ "id": self.id, "primaryAssetId": self.primary, "assets": assets })
    }
}

#[derive(Default)]
struct Fake {
    stacks: Vec<FakeStack>,
    posted: Vec<Vec<Uuid>>,
}

impl Fake {
    fn stack_of(&self, asset: Uuid) -> Option<&FakeStack> {
        self.stacks.iter().find(|s| s.members.contains(&asset))
    }

    fn create(&mut self, ids: Vec<Uuid>) -> FakeStack {
        let mut members = ids.clone();
        for stack in self.stacks.iter().filter(|s| ids.contains(&s.primary)) {
            members.extend(stack.members.iter().filter(|id| !ids.contains(id)));
        }
        self.stacks.retain(|s| !ids.contains(&s.primary));
        for stack in &mut self.stacks {
            stack.members.retain(|id| !ids.contains(id));
        }
        self.stacks.retain(|s| s.members.len() >= 2);
        let stack = FakeStack {
            id: Uuid::new_v4(),
            primary: ids[0],
            members,
        };
        self.stacks.push(stack.clone());
        self.posted.push(ids);
        stack
    }
}

type Shared = Arc<Mutex<Fake>>;

fn last_segment(req: &Request) -> Uuid {
    req.url
        .path()
        .rsplit('/')
        .next()
        .and_then(|id| id.parse().ok())
        .unwrap()
}

fn body_ids(req: &Request, key: &str) -> Value {
    serde_json::from_slice::<Value>(&req.body).unwrap()[key].clone()
}

async fn fake_immich(server: &MockServer, fake: &Shared) {
    let source = asset_id();
    let state = fake.clone();
    Mock::given(method("GET"))
        .and(path(format!("/api/assets/{source}")))
        .respond_with(move |_: &Request| {
            let mut detail = named_asset_detail(source, "DSC0001.ARW");
            if let Some(stack) = state.lock().unwrap().stack_of(source) {
                detail["stack"] = json!({
                    "id": stack.id,
                    "primaryAssetId": stack.primary,
                    "assetCount": stack.members.len()
                });
            }
            ResponseTemplate::new(200).set_body_json(detail)
        })
        .mount(server)
        .await;
    mock_jpeg_original(server, source).await;

    Mock::given(method("POST"))
        .and(path("/api/assets"))
        .respond_with(|_: &Request| {
            ResponseTemplate::new(201)
                .set_body_json(json!({ "id": Uuid::new_v4(), "status": "created" }))
        })
        .mount(server)
        .await;

    let state = fake.clone();
    Mock::given(method("GET"))
        .and(path_regex(STACK_PATH))
        .respond_with(move |req: &Request| {
            let id = last_segment(req);
            match state.lock().unwrap().stacks.iter().find(|s| s.id == id) {
                Some(stack) => ResponseTemplate::new(200).set_body_json(stack.to_json()),
                None => ResponseTemplate::new(404),
            }
        })
        .mount(server)
        .await;

    let state = fake.clone();
    Mock::given(method("POST"))
        .and(path("/api/stacks"))
        .respond_with(move |req: &Request| {
            let ids = serde_json::from_value(body_ids(req, "assetIds")).unwrap();
            let stack = state.lock().unwrap().create(ids);
            ResponseTemplate::new(201).set_body_json(stack.to_json())
        })
        .mount(server)
        .await;

    let state = fake.clone();
    Mock::given(method("PATCH"))
        .and(path_regex(STACK_PATH))
        .respond_with(move |req: &Request| {
            let id = last_segment(req);
            let primary = serde_json::from_value(body_ids(req, "primaryAssetId")).unwrap();
            let mut fake = state.lock().unwrap();
            match fake.stacks.iter_mut().find(|s| s.id == id) {
                Some(stack) => {
                    stack.primary = primary;
                    ResponseTemplate::new(200).set_body_json(stack.to_json())
                }
                None => ResponseTemplate::new(404),
            }
        })
        .mount(server)
        .await;
}

fn stacked(preset: &str) -> Value {
    json!({
        "filename_template": format!("{{name}}_{preset}"),
        "metadata": "none",
        "stack_with_original": true
    })
}

fn web_and_print() -> Value {
    json!({ "variants": [
        preset_variant("Web", stacked("Web")),
        preset_variant("Print", stacked("Print")),
    ] })
}

async fn start_job(
    fake: &Shared,
    params: Value,
) -> (MockServer, immich_edit_backend::state::AppState) {
    let server = MockServer::start().await;
    fake_immich(&server, fake).await;
    let state = test_state(&server).await;
    let token = seed_session(&server, &state).await;
    let app = wrap_auth(router(state.clone()), token);
    created_job_id(
        app.oneshot(create_job_request(
            "export_immich",
            params,
            &[asset_id().to_string()],
        ))
        .await
        .unwrap(),
    )
    .await;
    (server, state)
}

fn uploaded(results: &[Value], preset: &str) -> Uuid {
    let result = results
        .iter()
        .find(|r| r["preset"] == preset)
        .unwrap_or_else(|| panic!("no {preset} result in {results:?}"));
    assert_eq!(
        result["warnings"].as_array().map(Vec::len).unwrap_or(0),
        0,
        "{result}"
    );
    result["asset_id"].as_str().unwrap().parse().unwrap()
}

fn assert_one_stack(fake: &Shared, primary: Uuid, members: &[Uuid]) {
    let fake = fake.lock().unwrap();
    assert_eq!(fake.stacks.len(), 1, "{:?}", fake.posted);
    let stack = &fake.stacks[0];
    assert_eq!(stack.primary, primary);
    let mut expected = members.to_vec();
    let mut actual = stack.members.clone();
    expected.sort();
    actual.sort();
    assert_eq!(actual, expected);
}

#[tokio::test]
async fn a_second_preset_stacks_onto_the_first() {
    let fake = Shared::default();
    let (_server, state) = start_job(&fake, web_and_print()).await;

    let results = run_all(&state).await;

    let (web, print) = (uploaded(&results, "Web"), uploaded(&results, "Print"));
    let posted = fake.lock().unwrap().posted.clone();
    assert_eq!(
        posted,
        [vec![web, asset_id()], vec![web, print, asset_id()]]
    );
    assert_one_stack(&fake, web, &[asset_id(), web, print]);
}

#[tokio::test]
async fn the_first_preset_ends_on_top_when_it_finishes_last() {
    let fake = Shared::default();
    let (_server, state) = start_job(&fake, web_and_print()).await;
    let web_item = state.jobs.claim_next_item().await.unwrap().unwrap();
    let print_item = state.jobs.claim_next_item().await.unwrap().unwrap();
    assert_eq!((web_item.variant, print_item.variant), (0, 1));

    let print_result = run_item(&state, print_item).await;
    let web_result = run_item(&state, web_item).await;

    let results = [web_result, print_result];
    let (web, print) = (uploaded(&results, "Web"), uploaded(&results, "Print"));
    let posted = fake.lock().unwrap().posted.clone();
    assert_eq!(
        posted,
        [vec![print, asset_id()], vec![web, asset_id(), print]]
    );
    assert_one_stack(&fake, web, &[asset_id(), web, print]);
}

#[tokio::test]
async fn presets_finishing_together_share_one_stack() {
    let fake = Shared::default();
    let (_server, state) = start_job(&fake, web_and_print()).await;
    let web_item = state.jobs.claim_next_item().await.unwrap().unwrap();
    let print_item = state.jobs.claim_next_item().await.unwrap().unwrap();

    let (web_result, print_result) =
        tokio::join!(run_item(&state, web_item), run_item(&state, print_item));

    let results = [web_result, print_result];
    let (web, print) = (uploaded(&results, "Web"), uploaded(&results, "Print"));
    assert_one_stack(&fake, web, &[asset_id(), web, print]);
}

#[tokio::test]
async fn a_single_export_joins_an_existing_stack_on_top() {
    let fake = Shared::default();
    let other = Uuid::new_v4();
    fake.lock().unwrap().stacks.push(FakeStack {
        id: Uuid::new_v4(),
        primary: asset_id(),
        members: vec![asset_id(), other],
    });
    let (_server, state) = start_job(&fake, stacked("Web")).await;

    let results = run_all(&state).await;

    assert_eq!(results.len(), 1);
    assert!(results[0].get("preset").is_none(), "{}", results[0]);
    let edited: Uuid = results[0]["asset_id"].as_str().unwrap().parse().unwrap();
    let posted = fake.lock().unwrap().posted.clone();
    assert_eq!(posted, [vec![edited, asset_id(), other]]);
    assert_one_stack(&fake, edited, &[asset_id(), other, edited]);
}
