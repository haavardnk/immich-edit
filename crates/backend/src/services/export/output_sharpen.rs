use raw_pipeline::frame::{OutputSharpen, SharpenLevel, SharpenMedia};
use serde::Deserialize;

use crate::error::AppError;

const DEFAULT_PPI: u32 = 300;
const PPI_RANGE: std::ops::RangeInclusive<u32> = 72..=1200;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SharpenMediaOpt {
    Screen,
    Matte,
    Glossy,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SharpenAmountOpt {
    Low,
    #[default]
    Standard,
    High,
}

pub fn output_sharpen(
    media: Option<SharpenMediaOpt>,
    amount: SharpenAmountOpt,
    ppi: Option<u32>,
) -> Result<Option<OutputSharpen>, AppError> {
    let Some(media) = media else {
        return Ok(None);
    };
    let print_ppi = || {
        let ppi = ppi.unwrap_or(DEFAULT_PPI);
        if !PPI_RANGE.contains(&ppi) {
            return Err(AppError::BadRequest(format!(
                "output sharpening ppi must be between {} and {}",
                PPI_RANGE.start(),
                PPI_RANGE.end()
            )));
        }
        Ok(ppi)
    };
    let media = match media {
        SharpenMediaOpt::Screen if ppi.is_some() => {
            return Err(AppError::BadRequest(
                "screen output sharpening takes no ppi".into(),
            ));
        }
        SharpenMediaOpt::Screen => SharpenMedia::Screen,
        SharpenMediaOpt::Matte => SharpenMedia::Matte { ppi: print_ppi()? },
        SharpenMediaOpt::Glossy => SharpenMedia::Glossy { ppi: print_ppi()? },
    };
    let level = match amount {
        SharpenAmountOpt::Low => SharpenLevel::Low,
        SharpenAmountOpt::Standard => SharpenLevel::Standard,
        SharpenAmountOpt::High => SharpenLevel::High,
    };
    Ok(Some(OutputSharpen { media, level }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_media_amount_and_density() {
        assert_eq!(
            output_sharpen(None, SharpenAmountOpt::High, None).unwrap(),
            None
        );
        assert_eq!(
            output_sharpen(Some(SharpenMediaOpt::Screen), SharpenAmountOpt::Low, None).unwrap(),
            Some(OutputSharpen {
                media: SharpenMedia::Screen,
                level: SharpenLevel::Low
            })
        );
        assert_eq!(
            output_sharpen(
                Some(SharpenMediaOpt::Matte),
                SharpenAmountOpt::Standard,
                None
            )
            .unwrap(),
            Some(OutputSharpen {
                media: SharpenMedia::Matte { ppi: 300 },
                level: SharpenLevel::Standard
            })
        );
        assert_eq!(
            output_sharpen(
                Some(SharpenMediaOpt::Glossy),
                SharpenAmountOpt::High,
                Some(600)
            )
            .unwrap()
            .map(|s| s.media),
            Some(SharpenMedia::Glossy { ppi: 600 })
        );
    }

    #[test]
    fn rejects_bad_density() {
        for (media, ppi) in [
            (SharpenMediaOpt::Matte, 71),
            (SharpenMediaOpt::Glossy, 1201),
            (SharpenMediaOpt::Screen, 300),
        ] {
            assert!(
                output_sharpen(Some(media), SharpenAmountOpt::Standard, Some(ppi)).is_err(),
                "{media:?} {ppi}"
            );
        }
    }
}
