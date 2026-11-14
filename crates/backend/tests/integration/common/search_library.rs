use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{DateTime, SecondsFormat, Utc};
use serde_json::{Map, Value, json};
use uuid::Uuid;
use wiremock::matchers::{header, method, path_regex};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

#[derive(Clone, Copy, Debug, Default)]
pub struct Photo {
    pub id: Uuid,
    pub taken: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub favorite: bool,
    pub in_album: bool,
    pub tagged: bool,
}

pub const ALBUM: Uuid = Uuid::from_u128(0xa1b);
pub const TAG: Uuid = Uuid::from_u128(0x7a6);

const DEFAULT_SIZE: u64 = 250;

pub type Shelf = Arc<Mutex<Vec<Photo>>>;

type Condition = Box<dyn Fn(&Photo) -> bool>;

struct Search {
    conditions: Vec<Condition>,
    ascending: bool,
    size: usize,
    start: usize,
    structured: bool,
}

struct Library {
    photos: Shelf,
    delay: Duration,
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
            .lock()
            .unwrap()
            .iter()
            .copied()
            .filter(|photo| search.conditions.iter().all(|holds| holds(photo)))
            .collect();
        if request.url.path().ends_with("/statistics") {
            return ResponseTemplate::new(200)
                .set_body_json(json!({ "total": photos.len() }))
                .set_delay(self.delay);
        }
        sort(&mut photos, search.ascending);
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
        ResponseTemplate::new(200)
            .set_body_json(json!({
                "assets": {
                    "items": items, "count": items.len(), "total": photos.len(),
                    "nextPage": next_page, "nextCursor": next_cursor
                }
            }))
            .set_delay(self.delay)
    }
}

fn sort(photos: &mut [Photo], ascending: bool) {
    photos.sort_by_key(|photo| (photo.taken, photo.id));
    if !ascending {
        photos.reverse();
    }
}

fn summary(photo: &Photo) -> Value {
    json!({
        "id": photo.id,
        "type": "IMAGE",
        "originalFileName": "DSC0001.ARW",
        "fileCreatedAt": photo.taken.to_rfc3339_opts(SecondsFormat::Millis, true),
    })
}

fn size(body: &Map<String, Value>) -> usize {
    body.get("size")
        .and_then(Value::as_u64)
        .unwrap_or(DEFAULT_SIZE) as usize
}

fn flat(body: &Map<String, Value>) -> Option<Search> {
    let mut conditions: Vec<Condition> = Vec::new();
    for (key, value) in body {
        match (key.as_str(), value) {
            ("takenAfter", _) => conditions.push(compare("gte", value, taken)?),
            ("takenBefore", _) => conditions.push(compare("lte", value, taken)?),
            ("updatedAfter", _) => conditions.push(compare("gte", value, updated)?),
            ("isFavorite", Value::Bool(true)) => conditions.push(Box::new(|photo| photo.favorite)),
            ("albumIds" | "tagIds", _) => conditions.push(member(key, value)?),
            ("id", _) => conditions.push(id(value)?),
            ("order" | "size" | "page" | "withExif", _) => {}
            _ => return None,
        }
    }
    let size = size(body);
    let page = body.get("page").and_then(Value::as_u64).unwrap_or(1) as usize;
    Some(Search {
        conditions,
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
            "isFavorite" if *value == json!({ "eq": true }) => {
                conditions.push(Box::new(|photo| photo.favorite));
            }
            "id" => conditions.push(id(value.get("eq")?)?),
            "takenAt" => {
                for (op, bound) in value.as_object()? {
                    conditions.push(compare(op, bound, taken)?);
                }
            }
            "updatedAt" => {
                for (op, bound) in value.as_object()? {
                    conditions.push(compare(op, bound, updated)?);
                }
            }
            "albumIds" | "tagIds" => conditions.push(member(key, value.get("all")?)?),
            _ => return None,
        }
    }
    let order = body.get("orderBy");
    if order
        .and_then(|order| order.get("field"))
        .is_some_and(|field| field != "fileCreatedAt")
    {
        return None;
    }
    let start = match body.get("cursor") {
        None => 0,
        Some(cursor) => cursor.as_str()?.parse().ok()?,
    };
    Some(Search {
        conditions,
        ascending: order.and_then(|order| order.get("direction")) == Some(&json!("asc")),
        size: size(body),
        start,
        structured: true,
    })
}

fn id(value: &Value) -> Option<Condition> {
    let wanted: Uuid = value.as_str()?.parse().ok()?;
    Some(Box::new(move |photo| photo.id == wanted))
}

fn member(key: &str, ids: &Value) -> Option<Condition> {
    let condition: Condition = match key {
        "albumIds" if *ids == json!([ALBUM]) => Box::new(|photo| photo.in_album),
        "tagIds" if *ids == json!([TAG]) => Box::new(|photo| photo.tagged),
        _ => return None,
    };
    Some(condition)
}

fn taken(photo: &Photo) -> DateTime<Utc> {
    photo.taken
}

fn updated(photo: &Photo) -> DateTime<Utc> {
    photo.updated
}

fn compare(op: &str, bound: &Value, field: fn(&Photo) -> DateTime<Utc>) -> Option<Condition> {
    let bound = DateTime::parse_from_rfc3339(bound.as_str()?)
        .ok()?
        .with_timezone(&Utc);
    let condition: Condition = match op {
        "gte" => Box::new(move |photo| field(photo) >= bound),
        "gt" => Box::new(move |photo| field(photo) > bound),
        "lte" => Box::new(move |photo| field(photo) <= bound),
        "lt" => Box::new(move |photo| field(photo) < bound),
        _ => return None,
    };
    Some(condition)
}

pub fn feed(photos: &[Photo], ascending: bool) -> Vec<Uuid> {
    let mut sorted = photos.to_vec();
    sort(&mut sorted, ascending);
    sorted.iter().map(|photo| photo.id).collect()
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
