#[cfg(feature = "native")]
mod display;
mod rect;
#[cfg(feature = "native")]
mod retouch;
#[cfg(feature = "native")]
mod sensor;
#[cfg(feature = "native")]
pub(crate) mod support;

#[cfg(all(test, feature = "native"))]
mod tests;

#[cfg(feature = "native")]
pub use display::{DisplayLayout, DisplayTile};
pub use rect::Rect;
#[cfg(feature = "native")]
pub use retouch::StrokeReach;
#[cfg(feature = "native")]
pub(crate) use retouch::tile_strokes;
#[cfg(feature = "native")]
pub use sensor::{SensorChain, SensorTile, Warp};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TilePlan {
    SinglePass,
    SensorTiled { edge: u32 },
    DisplayTiled { edge: u32, held: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TilingReport {
    pub plan: TilePlan,
    pub tiles: u32,
}

impl TilePlan {
    pub fn display_edge(self) -> Option<u32> {
        match self {
            Self::DisplayTiled { edge, .. } => Some(edge),
            Self::SinglePass | Self::SensorTiled { .. } => None,
        }
    }
}

impl TilingReport {
    pub const SINGLE_PASS: Self = Self {
        plan: TilePlan::SinglePass,
        tiles: 1,
    };

    pub fn new(plan: TilePlan, tiles: usize) -> Self {
        Self {
            plan,
            tiles: tiles as u32,
        }
    }
}
