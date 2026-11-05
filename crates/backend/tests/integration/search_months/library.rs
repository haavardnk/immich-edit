use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{DateTime, SecondsFormat, Utc};
use serde_json::{Map, Value, json};
use uuid::Uuid;
use wiremock::matchers::{header, method, path_regex};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

#[derive(Clone, Copy)]
pub struct Photo {
    pub taken: DateTime<Utc>,
    pub favorite: bool,
}

pub type Shelf = Arc<Mutex<Vec<Photo>>>;

type Condition = Box<dyn Fn(&Photo) -> bool>;

struct Library {
    photos: Shelf,
    delay: Duration,
}

impl Respond for Library {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let Ok(Value::Object(body)) = request.body_json::<Value>() else {
            return ResponseTemplate::new(400);
        };
        let Some(conditions) = conditions(&body) else {
            return ResponseTemplate::new(400)
                .set_body_json(json!({ "message": format!("unexpected search {body:?}") }));
        };
        let mut photos: Vec<Photo> = self
            .photos
            .lock()
            .unwrap()
            .iter()
            .copied()
            .filter(|photo| conditions.iter().all(|holds| holds(photo)))
            .collect();
        if request.url.path().ends_with("/statistics") {
            return ResponseTemplate::new(200)
                .set_body_json(json!({ "total": photos.len() }))
                .set_delay(self.delay);
        }
        photos.sort_by_key(|photo| photo.taken);
        let direction = body
            .get("order")
            .or_else(|| body.get("orderBy").and_then(|order| order.get("direction")));
        if direction != Some(&json!("asc")) {
            photos.reverse();
        }
        let size = body.get("size").and_then(Value::as_u64).unwrap_or(250) as usize;
        let items: Vec<Value> = photos
            .iter()
            .take(size)
            .map(|photo| {
                json!({
                    "id": Uuid::new_v4(),
                    "type": "IMAGE",
                    "fileCreatedAt": photo.taken.to_rfc3339_opts(SecondsFormat::Millis, true),
                })
            })
            .collect();
        ResponseTemplate::new(200)
            .set_body_json(json!({
                "assets": {
                    "items": items, "count": items.len(), "total": items.len(),
                    "nextPage": null, "nextCursor": null
                }
            }))
            .set_delay(self.delay)
    }
}

fn conditions(body: &Map<String, Value>) -> Option<Vec<Condition>> {
    match body.get("filter") {
        Some(Value::Object(filter)) => structured(body, filter),
        Some(_) => None,
        None => flat(body),
    }
}

fn flat(body: &Map<String, Value>) -> Option<Vec<Condition>> {
    let mut conditions: Vec<Condition> = Vec::new();
    for (key, value) in body {
        match (key.as_str(), value) {
            ("takenAfter", _) => conditions.push(taken("gte", value)?),
            ("takenBefore", _) => conditions.push(taken("lte", value)?),
            ("isFavorite", Value::Bool(true)) => conditions.push(Box::new(|photo| photo.favorite)),
            ("order" | "size" | "page", _) => {}
            _ => return None,
        }
    }
    Some(conditions)
}

fn structured(body: &Map<String, Value>, filter: &Map<String, Value>) -> Option<Vec<Condition>> {
    if body
        .keys()
        .any(|key| !["filter", "orderBy", "size"].contains(&key.as_str()))
    {
        return None;
    }
    let mut conditions: Vec<Condition> = Vec::new();
    for (key, value) in filter {
        match key.as_str() {
            "trashedAt" if *value == json!({ "eq": null }) => {}
            "isFavorite" if *value == json!({ "eq": true }) => {
                conditions.push(Box::new(|photo| photo.favorite));
            }
            "takenAt" => {
                for (op, bound) in value.as_object()? {
                    conditions.push(taken(op, bound)?);
                }
            }
            _ => return None,
        }
    }
    Some(conditions)
}

fn taken(op: &str, bound: &Value) -> Option<Condition> {
    let bound = DateTime::parse_from_rfc3339(bound.as_str()?)
        .ok()?
        .with_timezone(&Utc);
    let condition: Condition = match op {
        "gte" => Box::new(move |photo| photo.taken >= bound),
        "gt" => Box::new(move |photo| photo.taken > bound),
        "lte" => Box::new(move |photo| photo.taken <= bound),
        "lt" => Box::new(move |photo| photo.taken < bound),
        _ => return None,
    };
    Some(condition)
}

pub fn at(raw: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(raw)
        .unwrap()
        .with_timezone(&Utc)
}

pub fn photos(raws: &[&str]) -> Vec<Photo> {
    raws.iter()
        .map(|raw| Photo {
            taken: at(raw),
            favorite: false,
        })
        .collect()
}

pub async fn mount(server: &MockServer, api_key: &str, photos: Vec<Photo>) -> Shelf {
    mount_slow(server, api_key, photos, Duration::ZERO).await
}

pub async fn mount_slow(
    server: &MockServer,
    api_key: &str,
    photos: Vec<Photo>,
    delay: Duration,
) -> Shelf {
    let shelf = Arc::new(Mutex::new(photos));
    Mock::given(method("POST"))
        .and(path_regex(r"^/api/search/(metadata|statistics)$"))
        .and(header("x-api-key", api_key))
        .respond_with(Library {
            photos: shelf.clone(),
            delay,
        })
        .mount(server)
        .await;
    shelf
}
