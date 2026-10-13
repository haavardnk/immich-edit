use little_exif::exif_tag::ExifTag;
use little_exif::metadata::Metadata;

use crate::frame::RawFrame;
use crate::math::smoothstep;

const NIGHT_EV: f32 = 0.0;
const DUSK_EV: f32 = 5.0;

pub(super) fn exposure_value(frame: &RawFrame) -> Option<f64> {
    let exif = frame.exif.as_ref()?;
    let time = rational(exif, &ExifTag::ExposureTime(Vec::new()))?;
    let aperture = rational(exif, &ExifTag::FNumber(Vec::new()))?;
    let iso = iso(exif)?;
    if time <= 0.0 || aperture <= 0.0 || iso <= 0.0 {
        return None;
    }
    Some((aperture * aperture / time).log2() - (iso / 100.0).log2())
}

pub(super) fn night(scene_ev: Option<f64>) -> f64 {
    scene_ev.map_or(0.0, |ev| {
        1.0 - f64::from(smoothstep(NIGHT_EV, DUSK_EV, ev as f32))
    })
}

fn rational(exif: &Metadata, tag: &ExifTag) -> Option<f64> {
    let value = match exif.get_tag(tag).next()? {
        ExifTag::ExposureTime(v) | ExifTag::FNumber(v) => v.first()?,
        _ => return None,
    };
    if value.denominator == 0 {
        return None;
    }
    Some(f64::from(value.nominator) / f64::from(value.denominator))
}

fn iso(exif: &Metadata) -> Option<f64> {
    if let Some(ExifTag::ISO(v)) = exif.get_tag(&ExifTag::ISO(Vec::new())).next() {
        return v.first().map(|&iso| f64::from(iso));
    }
    match exif.get_tag(&ExifTag::ISOSpeed(Vec::new())).next()? {
        ExifTag::ISOSpeed(v) => v.first().map(|&iso| f64::from(iso)),
        _ => None,
    }
}
