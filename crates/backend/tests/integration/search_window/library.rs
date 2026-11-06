use chrono::{DateTime, SecondsFormat, Utc};
use serde_json::{Map, Value, json};
use uuid::Uuid;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

use crate::common::TEST_API_KEY;

#[derive(Clone, Copy, Debug)]
pub struct Photo {
    pub id: Uuid,
    pub taken: DateTime<Utc>,
    pub bytes: u64,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Key {
    Taken,
    Size,
}

type Condition = Box<dyn Fn(&Photo) -> bool>;

struct Search {
    conditions: Vec<Condition>,
    key: Key,
    ascending: bool,
    size: usize,
    start: usize,
    structured: bool,
}

struct Library {
    photos: Vec<Photo>,
}

impl Respond for Library {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let Ok(Value::Object(body)) = request.body_json::<Value>() else {
            return ResponseTemplate::new(400);
        };
        let search = match body.get("filter") {
            Some(Value::Object(filter)) => structured(&body, filter),
            Some(_) => None,
            None => flat(&body),
        };
        let Some(search) = search else {
            return ResponseTemplate::new(400)
                .set_body_json(json!({ "message": format!("unexpected search {body:?}") }));
        };
        let mut photos: Vec<Photo> = self
            .photos
            .iter()
            .copied()
            .filter(|photo| search.conditions.iter().all(|holds| holds(photo)))
            .collect();
        photos.sort_by_key(|photo| sort_key(search.key, photo));
        if !search.ascending {
            photos.reverse();
        }
        let items: Vec<Value> = photos
            .iter()
            .skip(search.start)
            .take(search.size)
            .map(summary)
            .collect();
        let end = search.start + search.size;
        let more = end < photos.len();
        let (next_page, next_cursor) = match (more, search.structured) {
            (false, _) => (Value::Null, Value::Null),
            (true, true) => (Value::Null, json!(end.to_string())),
            (true, false) => (json!((end / search.size + 1).to_string()), Value::Null),
        };
        ResponseTemplate::new(200).set_body_json(json!({
            "assets": {
                "items": items, "count": items.len(), "total": photos.len(),
                "nextPage": next_page, "nextCursor": next_cursor
            }
        }))
    }
}

fn sort_key(key: Key, photo: &Photo) -> (i64, Uuid) {
    match key {
        Key::Taken => (photo.taken.timestamp_micros(), photo.id),
        Key::Size => (photo.bytes as i64, photo.id),
    }
}

fn summary(photo: &Photo) -> Value {
    json!({
        "id": photo.id,
        "type": "IMAGE",
        "originalFileName": "DSC0001.ARW",
        "fileCreatedAt": photo.taken.to_rfc3339_opts(SecondsFormat::Millis, true),
        "exifInfo": { "fileSizeInByte": photo.bytes },
    })
}

fn flat(body: &Map<String, Value>) -> Option<Search> {
    let mut conditions: Vec<Condition> = Vec::new();
    for (key, value) in body {
        match key.as_str() {
            "takenAfter" => conditions.push(taken("gte", value)?),
            "takenBefore" => conditions.push(taken("lte", value)?),
            "id" => conditions.push(id(value)?),
            "order" | "size" | "page" | "withExif" => {}
            _ => return None,
        }
    }
    let size = body.get("size")?.as_u64()? as usize;
    let page = body.get("page").and_then(Value::as_u64).unwrap_or(1) as usize;
    Some(Search {
        conditions,
        key: Key::Taken,
        ascending: body.get("order") == Some(&json!("asc")),
        size,
        start: (page - 1) * size,
        structured: false,
    })
}

fn structured(body: &Map<String, Value>, filter: &Map<String, Value>) -> Option<Search> {
    let allowed = ["filter", "orderBy", "size", "cursor", "withExif"];
    if body.keys().any(|key| !allowed.contains(&key.as_str())) {
        return None;
    }
    let mut conditions: Vec<Condition> = Vec::new();
    for (key, value) in filter {
        match key.as_str() {
            "trashedAt" if *value == json!({ "eq": null }) => {}
            "id" => conditions.push(id(value.get("eq")?)?),
            "takenAt" => {
                for (op, bound) in value.as_object()? {
                    conditions.push(taken(op, bound)?);
                }
            }
            "fileSizeInBytes" => {
                for (op, bound) in value.as_object()? {
                    conditions.push(bytes(op, bound)?);
                }
            }
            _ => return None,
        }
    }
    let order = body.get("orderBy");
    let key = match order.and_then(|order| order.get("field")) {
        None => Key::Taken,
        Some(field) if field == "fileCreatedAt" => Key::Taken,
        Some(field) if field == "fileSizeInBytes" => Key::Size,
        Some(_) => return None,
    };
    let ascending = order.and_then(|order| order.get("direction")) == Some(&json!("asc"));
    let start = match body.get("cursor") {
        None => 0,
        Some(cursor) => cursor.as_str()?.parse().ok()?,
    };
    Some(Search {
        conditions,
        key,
        ascending,
        size: body.get("size")?.as_u64()? as usize,
        start,
        structured: true,
    })
}

fn id(value: &Value) -> Option<Condition> {
    let wanted: Uuid = value.as_str()?.parse().ok()?;
    Some(Box::new(move |photo| photo.id == wanted))
}

fn taken(op: &str, bound: &Value) -> Option<Condition> {
    let bound = DateTime::parse_from_rfc3339(bound.as_str()?)
        .ok()?
        .with_timezone(&Utc);
    let condition: Condition = match op {
        "gte" => Box::new(move |photo| photo.taken >= bound),
        "lte" => Box::new(move |photo| photo.taken <= bound),
        "lt" => Box::new(move |photo| photo.taken < bound),
        _ => return None,
    };
    Some(condition)
}

fn bytes(op: &str, bound: &Value) -> Option<Condition> {
    let bound = bound.as_u64()?;
    let condition: Condition = match op {
        "gte" => Box::new(move |photo| photo.bytes >= bound),
        "lte" => Box::new(move |photo| photo.bytes <= bound),
        _ => return None,
    };
    Some(condition)
}

pub fn feed(photos: &[Photo], key: Key, ascending: bool) -> Vec<Uuid> {
    let mut sorted = photos.to_vec();
    sorted.sort_by_key(|photo| sort_key(key, photo));
    if !ascending {
        sorted.reverse();
    }
    sorted.iter().map(|photo| photo.id).collect()
}

pub async fn mount(server: &MockServer, photos: Vec<Photo>) {
    Mock::given(method("POST"))
        .and(path("/api/search/metadata"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(Library { photos })
        .mount(server)
        .await;
}
