#[cfg(feature = "native")]
mod display;
mod rect;
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
pub use sensor::{SensorChain, SensorTile, Warp};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TilePlan {
    SinglePass,
    SensorTiled { edge: u32 },
    DisplayTiled { edge: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TilingReport {
    pub plan: TilePlan,
    pub tiles: u32,
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
