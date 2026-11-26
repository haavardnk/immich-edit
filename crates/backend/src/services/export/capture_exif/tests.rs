use little_exif::exif_tag::ExifTag;
use serde_json::json;

use super::{Corrections, corrections};
use crate::immich::dto::AssetDetail;

fn asset(fields: serde_json::Value) -> AssetDetail {
    let mut base = json!({ "id": "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee" });
    if let (Some(base), Some(extra)) = (base.as_object_mut(), fields.as_object()) {
        base.extend(extra.clone());
    }
    serde_json::from_value(base).unwrap()
}

fn text(found: &Corrections, id: u16) -> Option<String> {
    found.overrides.iter().find_map(|t| match t {
        ExifTag::DateTimeOriginal(v)
        | ExifTag::CreateDate(v)
        | ExifTag::SubSecTimeOriginal(v)
        | ExifTag::OffsetTimeOriginal(v)
        | ExifTag::OffsetTimeDigitized(v)
        | ExifTag::ImageDescription(v)
            if t.as_u16() == id =>
        {
            Some(v.clone())
        }
        _ => None,
    })
}

fn latitude(found: &Corrections) -> Vec<(u32, u32)> {
    found
        .overrides
        .iter()
        .find_map(|t| match t {
            ExifTag::GPSLatitude(v) => {
                Some(v.iter().map(|p| (p.nominator, p.denominator)).collect())
            }
            _ => None,
        })
        .unwrap_or_default()
}

#[test]
fn a_zoned_photo_gets_its_wall_time_offset_and_sub_seconds() {
    let found = corrections(&asset(json!({
        "localDateTime": "2026-10-31T14:05:49.345Z",
        "exifInfo": {
            "dateTimeOriginal": "2026-10-31T13:05:49.345+00:00",
            "timeZone": "UTC+1",
            "description": "  Fløyen at dusk  "
        }
    })));
    assert_eq!(text(&found, 0x9003).as_deref(), Some("2026:10:31 14:05:49"));
    assert_eq!(text(&found, 0x9004).as_deref(), Some("2026:10:31 14:05:49"));
    assert_eq!(text(&found, 0x9291).as_deref(), Some("345"));
    assert_eq!(text(&found, 0x9011).as_deref(), Some("+01:00"));
    assert_eq!(text(&found, 0x9012).as_deref(), Some("+01:00"));
    assert_eq!(text(&found, 0x010e).as_deref(), Some("Fløyen at dusk"));
    assert!(found.cleared.contains(&0x9011) && !found.cleared.contains(&0x010e));
}

#[test]
fn a_zoneless_photo_clears_the_files_offset_and_writes_zero_sub_seconds() {
    let found = corrections(&asset(json!({
        "localDateTime": "2026-10-31T13:03:12.000Z",
        "exifInfo": { "dateTimeOriginal": "2026-10-31T13:03:12+00:00", "timeZone": null }
    })));
    assert_eq!(text(&found, 0x9291).as_deref(), Some("000"));
    assert_eq!(text(&found, 0x9011), None);
    assert!(found.cleared.contains(&0x9011));
    assert!(found.cleared.contains(&0x010e));
}

#[test]
fn offsets_round_to_the_minute_and_need_a_shown_time() {
    let rounded = corrections(&asset(json!({
        "localDateTime": "2026-03-01T08:30:00.000Z",
        "exifInfo": { "dateTimeOriginal": "2026-03-01T13:59:59.400+00:00", "timeZone": "UTC-5:30" }
    })));
    assert_eq!(text(&rounded, 0x9011).as_deref(), Some("-05:30"));
    let unshown = corrections(&asset(json!({
        "exifInfo": { "dateTimeOriginal": "2026-03-01T14:00:00+00:00", "timeZone": "UTC+1" }
    })));
    assert_eq!(text(&unshown, 0x9011), None);
    assert_eq!(
        text(&unshown, 0x9003).as_deref(),
        Some("2026:03:01 14:00:00")
    );
}

#[test]
fn coordinates_split_into_whole_minutes_and_seconds() {
    let found = corrections(&asset(
        json!({ "exifInfo": { "latitude": 9.1, "longitude": -70.5 } }),
    ));
    assert_eq!(latitude(&found), vec![(9, 1), (6, 1), (0, 1000)]);
    let south = corrections(&asset(
        json!({ "exifInfo": { "latitude": -33.8568, "longitude": 1.0 } }),
    ));
    assert_eq!(latitude(&south), vec![(33, 1), (51, 1), (24_480, 1000)]);
    assert!(
        south
            .overrides
            .iter()
            .any(|t| matches!(t, ExifTag::GPSLatitudeRef(r) if r == "S"))
    );
}

#[test]
fn a_long_description_is_cut_on_a_character_boundary() {
    let long = "ø".repeat(1500);
    let found = corrections(&asset(json!({ "exifInfo": { "description": long } })));
    let kept = text(&found, 0x010e).unwrap();
    assert!(kept.len() <= 2000 && kept.chars().all(|c| c == 'ø'));
}

#[test]
fn an_asset_without_exif_changes_nothing() {
    let found = corrections(&asset(json!({})));
    assert!(found.overrides.is_empty() && found.cleared.is_empty());
}
