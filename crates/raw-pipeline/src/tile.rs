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

#[cfg(feature = "native")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TilePlan {
    SinglePass,
    SensorTiled { edge: u32 },
    DisplayTiled { edge: u32 },
}
