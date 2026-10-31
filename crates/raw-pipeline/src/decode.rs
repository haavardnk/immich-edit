mod bad_pixels;
mod bitmap;
mod dng;
mod highlights;
mod mosaic;
mod raw;

#[cfg(test)]
mod tests;

use crate::PipelineError;
use crate::admission::over_budget;
use crate::frame::RawFrame;
use bitmap::{decode_image, sniff_format};
use dng::DngCorrections;
use little_exif::metadata::Metadata;
use raw::{decode_raw_fast, decode_raw_quality, frame_bytes};
use rawler::rawsource::RawSource;
use std::panic::{AssertUnwindSafe, catch_unwind};

type DevelopFn =
    fn(rawler::RawImage, Option<Metadata>, &DngCorrections) -> crate::PipelineResult<RawFrame>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Develop {
    Fast,
    Quality,
}

pub fn decode(data: &[u8]) -> crate::PipelineResult<RawFrame> {
    decode_within(data, Develop::Fast, None)
}

pub fn decode_quality(data: &[u8]) -> crate::PipelineResult<RawFrame> {
    decode_within(data, Develop::Quality, None)
}

pub fn decode_within(
    data: &[u8],
    develop: Develop,
    budget: Option<u64>,
) -> crate::PipelineResult<RawFrame> {
    let exif = crate::metadata::read(data);
    let source = RawSource::new_from_slice(data);
    let decoder = match rawler::get_decoder(&source) {
        Ok(decoder) => decoder,
        Err(err) => {
            return decode_image(data, exif, budget).map_err(|image_err| match image_err {
                PipelineError::OverBudget(_) => image_err,
                _ => PipelineError::Unsupported(format!(
                    "RAW format not supported by rawler ({}): {err}; image decode: {image_err}",
                    format_hint(data)
                )),
            });
        }
    };
    let develop_fn: DevelopFn = match develop {
        Develop::Fast => decode_raw_fast,
        Develop::Quality => decode_raw_quality,
    };
    let params = rawler::decoders::RawDecodeParams::default();
    catch_unwind(AssertUnwindSafe(|| {
        let raw_image = decoder
            .raw_image(&source, &params, false)
            .map_err(|e| PipelineError::Decode(format!("rawler: {e}")))?;
        admit_frame(frame_bytes(&raw_image, develop), budget)?;
        let fallback = catch_unwind(AssertUnwindSafe(|| decoder.raw_metadata(&source, &params)))
            .ok()
            .and_then(Result::ok)
            .map(|meta| {
                crate::metadata::exif::from_raw_metadata(&meta, &raw_image.make, &raw_image.model)
            });
        let exif = crate::metadata::exif::merge(exif, fallback);
        let corrections = DngCorrections::read(data);
        develop_fn(raw_image, exif, &corrections)
    }))
    .unwrap_or_else(|_| {
        Err(PipelineError::Unsupported(format!(
            "rawler panicked decoding ({})",
            format_hint(data)
        )))
    })
}

fn admit_frame(bytes: u64, budget: Option<u64>) -> crate::PipelineResult<()> {
    match budget {
        Some(budget) if bytes > budget => {
            Err(over_budget("decoding this photo", bytes, "memory", budget))
        }
        _ => Ok(()),
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
