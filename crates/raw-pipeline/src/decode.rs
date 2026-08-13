mod bitmap;
mod raw;

#[cfg(test)]
mod tests;

use crate::PipelineError;
use crate::frame::RawFrame;
use bitmap::{decode_image, sniff_format};
use raw::{decode_raw_fast, decode_raw_quality};

pub fn decode(data: &[u8]) -> crate::PipelineResult<RawFrame> {
    let exif = crate::exif::parse(data);
    let source = rawler::rawsource::RawSource::new_from_slice(data);
    let params = rawler::decoders::RawDecodeParams::default();
    match rawler::decode(&source, &params) {
        Ok(raw_image) => decode_raw_fast(raw_image, exif),
        Err(e) => raw_fallback_or_unsupported(e, data, exif),
    }
}

pub fn decode_quality(data: &[u8]) -> crate::PipelineResult<RawFrame> {
    let exif = crate::exif::parse(data);
    let source = rawler::rawsource::RawSource::new_from_slice(data);
    let params = rawler::decoders::RawDecodeParams::default();
    match rawler::decode(&source, &params) {
        Ok(raw_image) => decode_raw_quality(raw_image, exif),
        Err(e) => raw_fallback_or_unsupported(e, data, exif),
    }
}

fn raw_fallback_or_unsupported(
    err: impl std::fmt::Display,
    data: &[u8],
    exif: Option<little_exif::metadata::Metadata>,
) -> crate::PipelineResult<RawFrame> {
    let msg = format!("{err}");
    if msg.contains("No decoder found") {
        decode_image(data, exif)
    } else {
        Err(PipelineError::Unsupported(format!(
            "RAW format not supported by rawler ({}): {msg}",
            format_hint(data)
        )))
    }
}

fn format_hint(data: &[u8]) -> String {
    match sniff_format(data) {
        Some(f) => format!("{f:?}"),
        None => {
            let head: Vec<String> = data.iter().take(4).map(|b| format!("{b:02X}")).collect();
            if head.is_empty() {
                "empty".into()
            } else {
                format!("magic {}", head.join(" "))
            }
        }
    }
}
