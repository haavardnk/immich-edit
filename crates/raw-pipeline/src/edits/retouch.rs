use super::masks::{Vec2f, clamp_point};
use serde::{Deserialize, Serialize};

pub const N_MAX_RETOUCH_STROKES: usize = 64;
pub const N_MAX_RETOUCH_POINTS: usize = 256;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum RetouchMode {
    #[default]
    Heal,
    Clone,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetouchStroke {
    pub id: String,
    #[serde(default)]
    pub mode: RetouchMode,
    #[serde(default)]
    pub points: Vec<Vec2f>,
    pub radius: f32,
    #[serde(default)]
    pub hardness: f32,
    #[serde(default = "opacity_default")]
    pub opacity: f32,
    #[serde(default)]
    pub source: Vec2f,
    #[serde(default = "enabled_default")]
    pub enabled: bool,
}

fn opacity_default() -> f32 {
    1.0
}

fn enabled_default() -> bool {
    true
}

impl RetouchStroke {
    pub fn is_effective(&self) -> bool {
        self.enabled && !self.points.is_empty() && self.radius > 0.0 && self.opacity > 0.0
    }
}

pub(super) fn clamp_retouch(strokes: &[RetouchStroke]) -> Vec<RetouchStroke> {
    strokes
        .iter()
        .take(N_MAX_RETOUCH_STROKES)
        .map(|s| RetouchStroke {
            id: s.id.clone(),
            mode: s.mode,
            points: s
                .points
                .iter()
                .take(N_MAX_RETOUCH_POINTS)
                .map(|p| clamp_point(*p))
                .collect(),
            radius: s.radius.clamp(0.0, 1.0),
            hardness: s.hardness.clamp(0.0, 1.0),
            opacity: s.opacity.clamp(0.0, 1.0),
            source: clamp_point(s.source),
            enabled: s.enabled,
        })
        .collect()
}
