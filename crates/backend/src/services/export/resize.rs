use serde::Deserialize;

use crate::error::AppError;

use super::EXPORT_MAX_EDGE;

const MAX_PERCENT: f64 = 400.0;
const MAX_MEGAPIXELS: f64 = 1000.0;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResizeMode {
    Dimensions,
    Megapixels,
    Percent,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Resize {
    Fit {
        width: Option<u32>,
        height: Option<u32>,
        enlarge: bool,
    },
    Megapixels {
        megapixels: f64,
        enlarge: bool,
    },
    Percent(f64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutputEdge {
    pub max_edge: u32,
    pub enlarge: bool,
}

fn longest_within(cap: u32, long: u32, short: u32) -> u32 {
    let scaled_short = |edge: u32| (f64::from(short) * (f64::from(edge) / f64::from(long))).round();
    let bound = (f64::from(cap) + 0.5) * f64::from(long) / f64::from(short);
    let mut edge = (bound.ceil() as u32).saturating_sub(1);
    while edge > 1 && scaled_short(edge) > f64::from(cap) {
        edge -= 1;
    }
    edge
}

impl Resize {
    pub fn fit(width: Option<u32>, height: Option<u32>, enlarge: bool) -> Result<Self, AppError> {
        if width.is_none() && height.is_none() {
            return Err(AppError::BadRequest(
                "resize needs a width or a height".into(),
            ));
        }
        if [width, height]
            .into_iter()
            .flatten()
            .any(|edge| edge == 0 || edge > EXPORT_MAX_EDGE)
        {
            return Err(AppError::BadRequest(format!(
                "resize width and height must be from 1 to {EXPORT_MAX_EDGE}"
            )));
        }
        Ok(Self::Fit {
            width,
            height,
            enlarge,
        })
    }

    pub fn megapixels(megapixels: Option<f64>, enlarge: bool) -> Result<Self, AppError> {
        match megapixels {
            Some(value) if value.is_finite() && value > 0.0 && value <= MAX_MEGAPIXELS => {
                Ok(Self::Megapixels {
                    megapixels: value,
                    enlarge,
                })
            }
            _ => Err(AppError::BadRequest(format!(
                "resize megapixels must be above 0 and at most {MAX_MEGAPIXELS}"
            ))),
        }
    }

    pub fn percent(percent: Option<f64>) -> Result<Self, AppError> {
        match percent {
            Some(value) if value.is_finite() && (1.0..=MAX_PERCENT).contains(&value) => {
                Ok(Self::Percent(value))
            }
            _ => Err(AppError::BadRequest(format!(
                "resize percent must be from 1 to {MAX_PERCENT}"
            ))),
        }
    }

    pub fn output_edge(&self, crop: (u32, u32)) -> OutputEdge {
        let w = crop.0.max(1);
        let h = crop.1.max(1);
        let long = w.max(h);
        let (max_edge, enlarge) = match *self {
            Self::Percent(percent) => (
                (f64::from(long) * percent / 100.0).round() as u32,
                percent > 100.0,
            ),
            Self::Megapixels {
                megapixels,
                enlarge,
            } => (
                (megapixels * 1e6 * f64::from(long) / f64::from(w.min(h)))
                    .sqrt()
                    .round() as u32,
                enlarge,
            ),
            Self::Fit {
                width,
                height,
                enlarge,
            } => {
                let (long_cap, short_cap) = if w >= h {
                    (width, height)
                } else {
                    (height, width)
                };
                let from_short =
                    short_cap.map_or(EXPORT_MAX_EDGE, |cap| longest_within(cap, long, w.min(h)));
                (long_cap.unwrap_or(EXPORT_MAX_EDGE).min(from_short), enlarge)
            }
        };
        OutputEdge {
            max_edge: max_edge.clamp(1, EXPORT_MAX_EDGE),
            enlarge,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fit(width: Option<u32>, height: Option<u32>, crop: (u32, u32)) -> u32 {
        Resize::fit(width, height, false)
            .unwrap()
            .output_edge(crop)
            .max_edge
    }

    #[test]
    fn dimensions_fit_the_photo_inside_the_box() {
        for (width, height, crop, edge) in [
            (Some(2048), Some(2048), (6000, 4000), 2048),
            (Some(2048), Some(2048), (4000, 6000), 2048),
            (Some(2000), Some(1000), (6000, 4000), 1500),
            (Some(1080), None, (4000, 6000), 1620),
            (None, Some(1080), (6000, 4000), 1620),
            (Some(2000), Some(1333), (6000, 4000), 2000),
            (Some(1333), Some(2000), (4000, 6000), 2000),
            (None, Some(60000), (10000, 1000), EXPORT_MAX_EDGE),
        ] {
            assert_eq!(
                fit(width, height, crop),
                edge,
                "{width:?}x{height:?} {crop:?}"
            );
        }
    }

    #[test]
    fn a_linked_size_comes_out_exactly() {
        let crop = (7952, 5304);
        for width in (100..4000).step_by(7) {
            let height = (f64::from(width) * 5304.0 / 7952.0).round() as u32;
            let linked = fit(Some(width), Some(height), crop);
            assert_eq!(
                raw_pipeline::geom::scale_to_max(crop.0, crop.1, linked),
                (width, height)
            );
            let by_height = fit(None, Some(height), crop);
            assert_eq!(
                raw_pipeline::geom::scale_to_max(crop.0, crop.1, by_height).1,
                height
            );
        }
    }

    #[test]
    fn megapixels_keep_the_shape() {
        for (megapixels, crop, edge) in [
            (6.0, (6000, 4000), 3000),
            (6.0, (4000, 6000), 3000),
            (12.0, (4000, 4000), 3464),
        ] {
            assert_eq!(
                Resize::megapixels(Some(megapixels), false)
                    .unwrap()
                    .output_edge(crop)
                    .max_edge,
                edge,
                "{megapixels} {crop:?}"
            );
        }
    }

    #[test]
    fn percent_scales_and_enlarges_past_100() {
        for (percent, edge, enlarge) in [
            (50.0, 3000, false),
            (100.0, 6000, false),
            (150.0, 9000, true),
        ] {
            assert_eq!(
                Resize::percent(Some(percent))
                    .unwrap()
                    .output_edge((6000, 4000)),
                OutputEdge {
                    max_edge: edge,
                    enlarge
                }
            );
        }
    }

    #[test]
    fn enlarge_is_carried_through() {
        let resize = Resize::fit(Some(4000), None, true).unwrap();
        assert_eq!(
            resize.output_edge((1000, 500)),
            OutputEdge {
                max_edge: 4000,
                enlarge: true
            }
        );
    }

    #[test]
    fn rejects_values_out_of_range() {
        for (width, height) in [
            (None, None),
            (Some(0), None),
            (Some(2000), Some(EXPORT_MAX_EDGE + 1)),
        ] {
            assert!(
                Resize::fit(width, height, false).is_err(),
                "{width:?}x{height:?}"
            );
        }
        for percent in [None, Some(0.5), Some(401.0), Some(f64::NAN)] {
            assert!(Resize::percent(percent).is_err(), "{percent:?}");
        }
        for megapixels in [None, Some(0.0), Some(1001.0), Some(f64::INFINITY)] {
            assert!(
                Resize::megapixels(megapixels, false).is_err(),
                "{megapixels:?}"
            );
        }
    }
}
