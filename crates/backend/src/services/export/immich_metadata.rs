use std::time::Duration;

use serde_json::{Map, Value, json};
use uuid::Uuid;

use super::MetadataOpt;
use crate::immich::ImmichClient;
use crate::immich::dto::ExifInfo;

const EXTRACTION_WAIT: Duration = Duration::from_secs(30);
const MAX_POLL: Duration = Duration::from_secs(2);

pub async fn await_extraction(immich: &ImmichClient, id: Uuid) {
    let deadline = tokio::time::Instant::now() + EXTRACTION_WAIT;
    let mut delay = Duration::from_millis(100);
    while tokio::time::Instant::now() < deadline {
        match immich.asset(id).await {
            Ok(asset) if asset.width.is_none() => {}
            _ => return,
        }
        tokio::time::sleep(delay).await;
        delay = (delay * 2).min(MAX_POLL);
    }
    tracing::warn!(asset = %id, "Immich has not read the upload yet; copying metadata anyway");
}

pub fn bulk_update(new_id: Uuid, exif: Option<&ExifInfo>, mode: MetadataOpt) -> Option<Value> {
    if mode == MetadataOpt::None {
        return None;
    }
    let exif = exif?;
    let mut fields = Map::new();
    if let Some(date) = &exif.date_time_original {
        fields.insert("dateTimeOriginal".into(), json!(date));
    }
    if let Some(zone) = &exif.time_zone {
        fields.insert("timeZone".into(), json!(zone));
    }
    if mode == MetadataOpt::All
        && let (Some(latitude), Some(longitude)) = (exif.latitude, exif.longitude)
    {
        fields.insert("latitude".into(), json!(latitude));
        fields.insert("longitude".into(), json!(longitude));
    }
    if let Some(description) = exif.description.as_deref().filter(|d| !d.is_empty()) {
        fields.insert("description".into(), json!(description));
    }
    if let Some(rating) = exif.rating.filter(|r| (1..=5).contains(r)) {
        fields.insert("rating".into(), json!(rating));
    }
    if fields.is_empty() {
        return None;
    }
    fields.insert("ids".into(), json!([new_id]));
    Some(Value::Object(fields))
}
