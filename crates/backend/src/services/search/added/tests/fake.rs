use std::collections::HashMap;
use std::time::Duration;

use chrono::{DateTime, Datelike, SecondsFormat, TimeDelta, TimeZone, Utc};
use serde_json::{Value, json};
use url::Url;
use uuid::Uuid;
use wiremock::matchers::any;
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

use crate::immich::ImmichClient;
use crate::immich::client::ImmichAuth;
use crate::services::search::{SearchShape, Searcher};

pub const PARTNER: Uuid = Uuid::from_u128(0x5a);
pub const HIDDEN_PARTNER: Uuid = Uuid::from_u128(0x5b);

pub type Pick = fn(usize) -> bool;
pub type Query = HashMap<String, String>;

#[derive(Debug, Clone, Copy)]
pub struct Asset {
    pub at: DateTime<Utc>,
    pub id: Uuid,
    pub picked: bool,
    pub partner: bool,
}

struct FakeImmich {
    assets: Vec<Asset>,
    partners_readable: bool,
}

fn month(at: DateTime<Utc>) -> String {
    format!("{:04}-{:02}-01", at.year(), at.month())
}

impl FakeImmich {
    fn partners(&self) -> ResponseTemplate {
        if !self.partners_readable {
            return ResponseTemplate::new(403).set_body_json(json!({
                "message": "Missing required permission: partner.read",
                "statusCode": 403,
            }));
        }
        ResponseTemplate::new(200).set_body_json(json!([
            { "id": PARTNER, "name": "partner", "inTimeline": true },
            { "id": HIDDEN_PARTNER, "name": "hidden", "inTimeline": false },
        ]))
    }

    fn timeline(&self, request: &Request) -> ResponseTemplate {
        let query: Query = request.url.query_pairs().into_owned().collect();
        let get = |name: &str| query.get(name).map(String::as_str);
        let with_partners = get("withPartners") == Some("true");
        if get("orderBy") != Some("createdAt")
            || (with_partners
                && (get("isFavorite").is_some() || get("visibility") != Some("timeline")))
        {
            return ResponseTemplate::new(400);
        }
        let partner = match get("userId") {
            Some(id) if id == PARTNER.to_string() => Some(true),
            Some(_) => return ResponseTemplate::new(400),
            None if with_partners => None,
            None => Some(false),
        };
        let favorite = get("isFavorite").map(|value| value == "true");
        let mut assets: Vec<Asset> = self
            .assets
            .iter()
            .filter(|asset| partner.is_none_or(|partner| asset.partner == partner))
            .filter(|asset| favorite.is_none_or(|favorite| asset.picked == favorite))
            .copied()
            .collect();
        assets.sort_by_key(|asset| (asset.at, asset.id));
        if get("order") != Some("asc") {
            assets.reverse();
        }
        let Some(bucket) = get("timeBucket") else {
            let mut months: Vec<(String, u64)> = Vec::new();
            for asset in &assets {
                match months.last_mut() {
                    Some((key, count)) if *key == month(asset.at) => *count += 1,
                    _ => months.push((month(asset.at), 1)),
                }
            }
            let body: Vec<Value> = months
                .into_iter()
                .map(|(key, count)| json!({ "timeBucket": key, "count": count }))
                .collect();
            return ResponseTemplate::new(200).set_body_json(body);
        };
        let (created, ids): (Vec<String>, Vec<Uuid>) = assets
            .iter()
            .filter(|asset| month(asset.at) == bucket)
            .map(|asset| {
                let created = asset.at.naive_utc().format("%Y-%m-%dT%H:%M:%S%.6f");
                (created.to_string(), asset.id)
            })
            .unzip();
        ResponseTemplate::new(200).set_body_json(json!({ "id": ids, "createdAt": created }))
    }

    fn search(&self, request: &Request) -> ResponseTemplate {
        let body: Value = request.body_json().unwrap();
        let bound = |name: &str| {
            body.get(name)
                .and_then(Value::as_str)
                .map(|s| DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc))
        };
        let after = bound("createdAfter");
        let before = bound("createdBefore");
        let favorite = body.get("isFavorite").and_then(Value::as_bool);
        let images = body.get("type").and_then(Value::as_str) == Some("IMAGE");
        let id = body.get("id").and_then(Value::as_str);
        let items: Vec<Value> = self
            .assets
            .iter()
            .filter(|asset| after.is_none_or(|after| asset.at >= after))
            .filter(|asset| before.is_none_or(|before| asset.at <= before))
            .filter(|asset| favorite.is_none_or(|favorite| asset.picked == favorite))
            .filter(|asset| !images || asset.picked)
            .filter(|asset| id.is_none_or(|id| asset.id.to_string() == id))
            .map(|asset| {
                json!({
                    "id": asset.id,
                    "type": if asset.picked { "IMAGE" } else { "VIDEO" },
                    "isFavorite": asset.picked,
                    "createdAt": asset.at.to_rfc3339_opts(SecondsFormat::Micros, true),
                })
            })
            .collect();
        ResponseTemplate::new(200).set_body_json(json!({
            "assets": { "items": items, "count": items.len(), "total": items.len(), "nextPage": null },
        }))
    }
}

impl Respond for FakeImmich {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        match request.url.path() {
            "/api/search/metadata" => self.search(request),
            "/api/partners" => self.partners(),
            _ => self.timeline(request),
        }
    }
}

pub fn library(count: usize, pick: Pick) -> Vec<Asset> {
    let base = Utc.with_ymd_and_hms(2024, 1, 20, 8, 0, 0).unwrap();
    (0..count)
        .map(|i| Asset {
            at: base
                + TimeDelta::days(2 * i as i64)
                + TimeDelta::microseconds(1_234_567 * i as i64),
            id: Uuid::new_v4(),
            picked: pick(i),
            partner: false,
        })
        .collect()
}

pub async fn serve(assets: &[Asset], partners_readable: bool) -> (MockServer, Searcher) {
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(FakeImmich {
            assets: assets.to_vec(),
            partners_readable,
        })
        .mount(&server)
        .await;
    let immich = ImmichClient::with_auth(
        Url::parse(&server.uri()).unwrap(),
        ImmichAuth::ApiKey("key".into()),
        Duration::from_secs(5),
        None,
    )
    .unwrap();
    (server, Searcher::new(immich, SearchShape::Flat))
}

pub async fn requests(server: &MockServer, path: &str) -> Vec<Query> {
    server
        .received_requests()
        .await
        .unwrap_or_default()
        .iter()
        .filter(|request| request.url.path() == path)
        .map(|request| request.url.query_pairs().into_owned().collect())
        .collect()
}
