use std::ops::RangeInclusive;
use std::sync::Arc;

use raw_pipeline::frame::{Align, Watermark, WatermarkAnchor, WatermarkImage};
use serde::Deserialize;

use crate::error::AppError;

pub const DEFAULT_SIZE: f32 = 0.2;
pub const DEFAULT_OPACITY: f32 = 0.8;
pub const DEFAULT_INSET: f32 = 0.03;
const SIZE_RANGE: RangeInclusive<f32> = 0.01..=1.0;
const OPACITY_RANGE: RangeInclusive<f32> = 0.01..=1.0;
const INSET_RANGE: RangeInclusive<f32> = 0.0..=0.5;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum WatermarkAnchorOpt {
    TopLeft,
    Top,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    Bottom,
    #[default]
    BottomRight,
}

impl WatermarkAnchorOpt {
    fn anchor(self) -> WatermarkAnchor {
        let (x, y) = match self {
            Self::TopLeft => (Align::Start, Align::Start),
            Self::Top => (Align::Center, Align::Start),
            Self::TopRight => (Align::End, Align::Start),
            Self::Left => (Align::Start, Align::Center),
            Self::Center => (Align::Center, Align::Center),
            Self::Right => (Align::End, Align::Center),
            Self::BottomLeft => (Align::Start, Align::End),
            Self::Bottom => (Align::Center, Align::End),
            Self::BottomRight => (Align::End, Align::End),
        };
        WatermarkAnchor { x, y }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WatermarkPlacement {
    pub id: String,
    size: f32,
    opacity: f32,
    anchor: WatermarkAnchor,
    inset: f32,
}

impl WatermarkPlacement {
    pub fn with_image(self, image: Arc<WatermarkImage>) -> Watermark {
        Watermark {
            image,
            size: self.size,
            opacity: self.opacity,
            anchor: self.anchor,
            inset: self.inset,
        }
    }
}

fn fraction(name: &str, value: f32, range: RangeInclusive<f32>) -> Result<f32, AppError> {
    if range.contains(&value) {
        return Ok(value);
    }
    Err(AppError::BadRequest(format!(
        "{name} must be between {} and {}",
        range.start(),
        range.end()
    )))
}

pub fn watermark_placement(
    id: Option<&str>,
    size: f32,
    opacity: f32,
    anchor: WatermarkAnchorOpt,
    inset: f32,
) -> Result<Option<WatermarkPlacement>, AppError> {
    let Some(id) = id else {
        return Ok(None);
    };
    Ok(Some(WatermarkPlacement {
        id: id.to_string(),
        size: fraction("watermark_size", size, SIZE_RANGE)?,
        opacity: fraction("watermark_opacity", opacity, OPACITY_RANGE)?,
        anchor: anchor.anchor(),
        inset: fraction("watermark_inset", inset, INSET_RANGE)?,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_anchor_and_fractions() {
        assert_eq!(
            watermark_placement(None, 5.0, 5.0, WatermarkAnchorOpt::Top, 5.0).unwrap(),
            None
        );
        let placement =
            watermark_placement(Some("wm"), 0.3, 0.5, WatermarkAnchorOpt::Left, 0.0).unwrap();
        assert_eq!(
            placement,
            Some(WatermarkPlacement {
                id: "wm".into(),
                size: 0.3,
                opacity: 0.5,
                anchor: WatermarkAnchor {
                    x: Align::Start,
                    y: Align::Center
                },
                inset: 0.0,
            })
        );
    }

    #[test]
    fn rejects_out_of_range_fractions() {
        for (size, opacity, inset) in [
            (0.0, 0.8, 0.03),
            (1.5, 0.8, 0.03),
            (0.2, 0.0, 0.03),
            (0.2, 1.1, 0.03),
            (0.2, 0.8, -0.1),
            (0.2, 0.8, 0.6),
            (f32::NAN, 0.8, 0.03),
        ] {
            assert!(
                watermark_placement(
                    Some("wm"),
                    size,
                    opacity,
                    WatermarkAnchorOpt::default(),
                    inset
                )
                .is_err(),
                "{size} {opacity} {inset}"
            );
        }
    }
}
