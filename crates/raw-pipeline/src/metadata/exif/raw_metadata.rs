use little_exif::exif_tag::ExifTag;
use little_exif::metadata::Metadata;
use little_exif::rational::{iR64, uR64};
use rawler::decoders::RawMetadata;
use rawler::exif::ExifGPS;
use rawler::formats::tiff::{Rational, SRational};

pub fn from_raw_metadata(meta: &RawMetadata, make: &str, model: &str) -> Metadata {
    let exif = &meta.exif;
    let lens_make = exif
        .lens_make
        .clone()
        .or_else(|| meta.lens.as_ref().map(|l| l.lens_make.clone()));
    let lens_model = exif
        .lens_model
        .clone()
        .or_else(|| meta.lens.as_ref().map(|l| l.lens_model.clone()));
    let camera = [
        text(make).map(ExifTag::Make),
        text(model).map(ExifTag::Model),
        string(&exif.artist, ExifTag::Artist),
        string(&exif.copyright, ExifTag::Copyright),
        string(&exif.owner_name, ExifTag::OwnerName),
        string(&exif.serial_number, ExifTag::SerialNumber),
        string(&lens_make, ExifTag::LensMake),
        string(&lens_model, ExifTag::LensModel),
        string(&exif.lens_serial_number, ExifTag::LensSerialNumber),
        exif.lens_spec
            .map(|v| ExifTag::LensInfo(v.map(unsigned).to_vec())),
        string(&exif.date_time_original, ExifTag::DateTimeOriginal),
        string(&exif.create_date, ExifTag::CreateDate),
        string(&exif.modify_date, ExifTag::ModifyDate),
        string(&exif.offset_time, ExifTag::OffsetTime),
        string(&exif.offset_time_original, ExifTag::OffsetTimeOriginal),
        string(&exif.offset_time_digitized, ExifTag::OffsetTimeDigitized),
        string(&exif.sub_sec_time, ExifTag::SubSecTime),
        string(&exif.sub_sec_time_original, ExifTag::SubSecTimeOriginal),
        string(&exif.sub_sec_time_digitized, ExifTag::SubSecTimeDigitized),
    ];
    let exposure = [
        rational(exif.exposure_time, ExifTag::ExposureTime),
        rational(exif.fnumber, ExifTag::FNumber),
        rational(exif.aperture_value, ExifTag::ApertureValue),
        rational(exif.max_aperture_value, ExifTag::MaxApertureValue),
        rational(exif.focal_length, ExifTag::FocalLength),
        rational(exif.subject_distance, ExifTag::SubjectDistance),
        rational(exif.flash_energy, ExifTag::FlashEnergy),
        signed(exif.shutter_speed_value, ExifTag::ShutterSpeedValue),
        signed(exif.brightness_value, ExifTag::BrightnessValue),
        signed(exif.exposure_bias, ExifTag::ExposureCompensation),
        exif.iso_speed_ratings.map(|v| ExifTag::ISO(vec![v])),
        exif.iso_speed.map(|v| ExifTag::ISOSpeed(vec![v])),
        exif.recommended_exposure_index
            .map(|v| ExifTag::RecommendedExposureIndex(vec![v])),
        short(exif.sensitivity_type, ExifTag::SensitivityType),
        short(exif.exposure_program, ExifTag::ExposureProgram),
        short(exif.exposure_mode, ExifTag::ExposureMode),
        short(exif.metering_mode, ExifTag::MeteringMode),
        short(exif.light_source, ExifTag::LightSource),
        short(exif.flash, ExifTag::Flash),
        short(exif.white_balance, ExifTag::WhiteBalance),
        short(exif.scene_capture_type, ExifTag::SceneCaptureType),
        short(exif.subject_distance_range, ExifTag::SubjectDistanceRange),
    ];
    let gps = exif.gps.as_ref().map(gps_tags).unwrap_or_default();
    let mut out = Metadata::new();
    camera
        .into_iter()
        .chain(exposure)
        .chain(gps)
        .flatten()
        .for_each(|t| out.set_tag(t));
    out
}

fn gps_tags(gps: &ExifGPS) -> Vec<Option<ExifTag>> {
    vec![
        gps.gps_version_id
            .map(|v| ExifTag::GPSVersionID(v.to_vec())),
        string(&gps.gps_latitude_ref, ExifTag::GPSLatitudeRef),
        rationals(gps.gps_latitude, ExifTag::GPSLatitude),
        string(&gps.gps_longitude_ref, ExifTag::GPSLongitudeRef),
        rationals(gps.gps_longitude, ExifTag::GPSLongitude),
        gps.gps_altitude_ref
            .map(|v| ExifTag::GPSAltitudeRef(vec![v])),
        rational(gps.gps_altitude, ExifTag::GPSAltitude),
        rationals(gps.gps_timestamp, ExifTag::GPSTimeStamp),
        string(&gps.gps_satellites, ExifTag::GPSSatellites),
        string(&gps.gps_status, ExifTag::GPSStatus),
        string(&gps.gps_measure_mode, ExifTag::GPSMeasureMode),
        rational(gps.gps_dop, ExifTag::GPSDOP),
        string(&gps.gps_speed_ref, ExifTag::GPSSpeedRef),
        rational(gps.gps_speed, ExifTag::GPSSpeed),
        string(&gps.gps_track_ref, ExifTag::GPSTrackRef),
        rational(gps.gps_track, ExifTag::GPSTrack),
        string(&gps.gps_img_direction_ref, ExifTag::GPSImgDirectionRef),
        rational(gps.gps_img_direction, ExifTag::GPSImgDirection),
        string(&gps.gps_map_datum, ExifTag::GPSMapDatum),
        string(&gps.gps_dest_latitude_ref, ExifTag::GPSDestLatitudeRef),
        rationals(gps.gps_dest_latitude, ExifTag::GPSDestLatitude),
        string(&gps.gps_dest_longitude_ref, ExifTag::GPSDestLongitudeRef),
        rationals(gps.gps_dest_longitude, ExifTag::GPSDestLongitude),
        string(&gps.gps_dest_bearing_ref, ExifTag::GPSDestBearingRef),
        rational(gps.gps_dest_bearing, ExifTag::GPSDestBearing),
        string(&gps.gps_dest_distance_ref, ExifTag::GPSDestDistanceRef),
        rational(gps.gps_dest_distance, ExifTag::GPSDestDistance),
        gps.gps_processing_method
            .clone()
            .map(ExifTag::GPSProcessingMethod),
        gps.gps_area_information
            .clone()
            .map(ExifTag::GPSAreaInformation),
        string(&gps.gps_date_stamp, ExifTag::GPSDateStamp),
        short(gps.gps_differential, ExifTag::GPSDifferential),
        rational(gps.gps_h_positioning_error, ExifTag::GPSHPositioningError),
    ]
}

fn text(value: &str) -> Option<String> {
    let trimmed = value.trim_matches(|c: char| c.is_whitespace() || c == '\0');
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn string(value: &Option<String>, tag: fn(String) -> ExifTag) -> Option<ExifTag> {
    value.as_deref().and_then(text).map(tag)
}

fn short(value: Option<u16>, tag: fn(Vec<u16>) -> ExifTag) -> Option<ExifTag> {
    value.map(|v| tag(vec![v]))
}

fn unsigned(value: Rational) -> uR64 {
    uR64 {
        nominator: value.n,
        denominator: value.d,
    }
}

fn rational(value: Option<Rational>, tag: fn(Vec<uR64>) -> ExifTag) -> Option<ExifTag> {
    value.map(|v| tag(vec![unsigned(v)]))
}

fn rationals(value: Option<[Rational; 3]>, tag: fn(Vec<uR64>) -> ExifTag) -> Option<ExifTag> {
    value.map(|v| tag(v.map(unsigned).to_vec()))
}

fn signed(value: Option<SRational>, tag: fn(Vec<iR64>) -> ExifTag) -> Option<ExifTag> {
    value.map(|v| {
        tag(vec![iR64 {
            nominator: v.n,
            denominator: v.d,
        }])
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rawler::exif::Exif;

    fn ids(meta: &Metadata) -> Vec<u16> {
        meta.into_iter().map(|t| t.as_u16()).collect()
    }

    #[test]
    fn maps_capture_camera_lens_and_gps() {
        let meta = RawMetadata {
            exif: Exif {
                date_time_original: Some("2017:01:14 17:43:41".into()),
                offset_time_original: Some("+01:00".into()),
                lens_model: Some("XF35mmF1.4 R".into()),
                exposure_bias: Some(SRational { n: -2, d: 3 }),
                iso_speed_ratings: Some(400),
                gps: Some(ExifGPS {
                    gps_latitude_ref: Some("N".into()),
                    gps_latitude: Some([Rational { n: 59, d: 1 }; 3]),
                    ..ExifGPS::default()
                }),
                ..Exif::default()
            },
            model: "X-T2".into(),
            make: "Fujifilm".into(),
            lens: None,
            unique_image_id: None,
            rating: None,
        };

        let out = from_raw_metadata(&meta, "FUJIFILM", "X-T2\0 ");

        let has_make = out
            .into_iter()
            .any(|t| matches!(t, ExifTag::Make(v) if v == "FUJIFILM"));
        let has_model = out
            .into_iter()
            .any(|t| matches!(t, ExifTag::Model(v) if v == "X-T2"));
        let missing: Vec<u16> = [0x9003, 0x9011, 0xa434, 0x9204, 0x8827, 0x0001, 0x0002]
            .into_iter()
            .filter(|id| !ids(&out).contains(id))
            .collect();
        if !has_make || !has_model || !missing.is_empty() {
            panic!("make {has_make}, model {has_model}, missing {missing:04x?}");
        }
    }
}
