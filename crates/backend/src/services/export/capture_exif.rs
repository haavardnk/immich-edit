use chrono::{DateTime, FixedOffset, Timelike};
use little_exif::exif_tag::ExifTag;
use little_exif::rational::uR64;

use crate::immich::dto::AssetDetail;

const ARC_MILLIS: u64 = 1000;
const MAX_OFFSET_MINUTES: i64 = 14 * 60;
const MAX_DESCRIPTION_BYTES: usize = 2000;
const TIME_TAGS: [u16; 6] = [0x9003, 0x9004, 0x9011, 0x9012, 0x9291, 0x9292];
const DESCRIPTION_TAG: u16 = 0x010e;

#[derive(Debug, Default)]
pub struct Corrections {
    pub overrides: Vec<ExifTag>,
    pub cleared: Vec<u16>,
}

fn parsed(value: Option<&str>) -> Option<DateTime<FixedOffset>> {
    value.and_then(|v| DateTime::parse_from_rfc3339(v).ok())
}

fn offset_text(minutes: i64) -> String {
    let sign = if minutes < 0 { '-' } else { '+' };
    let total = minutes.abs();
    format!("{sign}{:02}:{:02}", total / 60, total % 60)
}

fn offset_minutes(wall: DateTime<FixedOffset>, original: DateTime<FixedOffset>) -> Option<i64> {
    let seconds = (wall.naive_utc() - original.naive_utc()).num_seconds();
    let minutes = (seconds as f64 / 60.0).round() as i64;
    (minutes.abs() <= MAX_OFFSET_MINUTES).then_some(minutes)
}

fn taken(asset: &AssetDetail, into: &mut Corrections) {
    let exif = asset.exif_info.as_ref();
    let original = parsed(exif.and_then(|e| e.date_time_original.as_deref()));
    let local = parsed(asset.local_date_time.as_deref());
    let Some(wall) = local.or(original) else {
        return;
    };
    into.cleared.extend(TIME_TAGS);
    let stamp = wall.naive_utc().format("%Y:%m:%d %H:%M:%S").to_string();
    let millis = format!("{:03}", wall.nanosecond() / 1_000_000);
    into.overrides.extend([
        ExifTag::DateTimeOriginal(stamp.clone()),
        ExifTag::CreateDate(stamp),
        ExifTag::SubSecTimeOriginal(millis.clone()),
        ExifTag::SubSecTimeDigitized(millis),
    ]);
    let zoned = exif.is_some_and(|e| e.time_zone.as_deref().is_some_and(|z| !z.is_empty()));
    let offset = match (zoned, local, original) {
        (true, Some(local), Some(original)) => offset_minutes(local, original),
        _ => None,
    };
    if let Some(minutes) = offset {
        let text = offset_text(minutes);
        into.overrides.extend([
            ExifTag::OffsetTimeOriginal(text.clone()),
            ExifTag::OffsetTimeDigitized(text),
        ]);
    }
}

fn dms(value: f64) -> Vec<uR64> {
    let total = (value.abs() * 3600.0 * ARC_MILLIS as f64).round() as u64;
    let per_degree = 3600 * ARC_MILLIS;
    let per_minute = 60 * ARC_MILLIS;
    vec![
        uR64 {
            nominator: (total / per_degree) as u32,
            denominator: 1,
        },
        uR64 {
            nominator: ((total % per_degree) / per_minute) as u32,
            denominator: 1,
        },
        uR64 {
            nominator: (total % per_minute) as u32,
            denominator: ARC_MILLIS as u32,
        },
    ]
}

fn location(asset: &AssetDetail, into: &mut Corrections) {
    let Some(exif) = asset.exif_info.as_ref() else {
        return;
    };
    let (Some(lat), Some(lon)) = (exif.latitude, exif.longitude) else {
        return;
    };
    if !(lat.is_finite() && lon.is_finite() && lat.abs() <= 90.0 && lon.abs() <= 180.0) {
        return;
    }
    into.overrides.extend([
        ExifTag::GPSLatitudeRef(if lat < 0.0 { "S" } else { "N" }.into()),
        ExifTag::GPSLatitude(dms(lat)),
        ExifTag::GPSLongitudeRef(if lon < 0.0 { "W" } else { "E" }.into()),
        ExifTag::GPSLongitude(dms(lon)),
    ]);
}

fn capped(text: &str) -> &str {
    if text.len() <= MAX_DESCRIPTION_BYTES {
        return text;
    }
    let end = (0..=MAX_DESCRIPTION_BYTES)
        .rev()
        .find(|&i| text.is_char_boundary(i))
        .unwrap_or(0);
    &text[..end]
}

fn description(asset: &AssetDetail, into: &mut Corrections) {
    let Some(exif) = asset.exif_info.as_ref() else {
        return;
    };
    match exif.description.as_deref().map(str::trim) {
        Some(text) if !text.is_empty() => {
            into.overrides
                .push(ExifTag::ImageDescription(capped(text).to_string()));
        }
        _ => into.cleared.push(DESCRIPTION_TAG),
    }
}

pub fn corrections(asset: &AssetDetail) -> Corrections {
    let mut into = Corrections::default();
    taken(asset, &mut into);
    location(asset, &mut into);
    description(asset, &mut into);
    into
}

#[cfg(test)]
mod tests;
