mod bitmap;
mod raw;

#[cfg(test)]
mod tests;

use crate::PipelineError;
use crate::frame::RawFrame;
use bitmap::{decode_image, sniff_format};
use little_exif::metadata::Metadata;
use raw::{decode_raw_fast, decode_raw_quality};
use rawler::rawsource::RawSource;
use std::panic::{AssertUnwindSafe, catch_unwind};

type DevelopFn = fn(rawler::RawImage, Option<Metadata>) -> crate::PipelineResult<RawFrame>;

pub fn decode(data: &[u8]) -> crate::PipelineResult<RawFrame> {
    decode_with(data, decode_raw_fast)
}

pub fn decode_quality(data: &[u8]) -> crate::PipelineResult<RawFrame> {
    decode_with(data, decode_raw_quality)
}

fn decode_with(data: &[u8], develop: DevelopFn) -> crate::PipelineResult<RawFrame> {
    let exif = crate::exif::parse(data);
    let source = RawSource::new_from_slice(data);
    if let Err(err) = rawler::get_decoder(&source) {
        return decode_image(data, exif).map_err(|_| {
            PipelineError::Unsupported(format!(
                "RAW format not supported by rawler ({}): {err}",
                format_hint(data)
            ))
        });
    }
    let params = rawler::decoders::RawDecodeParams::default();
    catch_unwind(AssertUnwindSafe(|| {
        let raw_image = rawler::decode(&source, &params)
            .map_err(|e| PipelineError::Decode(format!("rawler: {e}")))?;
        develop(raw_image, exif)
    }))
    .unwrap_or_else(|_| {
        Err(PipelineError::Unsupported(format!(
            "rawler panicked decoding ({})",
            format_hint(data)
        )))
    })
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
