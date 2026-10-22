#[cfg(feature = "native")]
mod bake;
#[cfg(feature = "native")]
mod blend;
#[cfg(feature = "native")]
mod low_frequency;
#[cfg(feature = "native")]
mod model_space;
mod plane;
#[cfg(feature = "native")]
mod tiles;
#[cfg(feature = "native")]
mod wire;

#[cfg(test)]
use crate::frame::{FrameId, FrameMeta, RawFrame};

#[cfg(feature = "native")]
pub use bake::bake;
#[cfg(feature = "native")]
pub(crate) use blend::{DenoiseMix, active_plane, blend, hash_active};
#[cfg(feature = "native")]
pub use model_space::ModelSpace;
pub use plane::DenoisePlane;
#[cfg(feature = "native")]
pub use wire::{decode, encode};

#[cfg(test)]
fn raw_frame(width: usize, height: usize, data: Vec<f32>) -> RawFrame {
    RawFrame {
        meta: FrameMeta {
            width,
            height,
            wb_coeffs: [1.0; 4],
            xyz_to_cam: [[0.0; 3]; 4],
            color_matrices: Vec::new(),
            orientation: (false, false, false),
            is_raw: true,
            capture_sigma: None,
            model: String::new(),
        },
        cfa_pattern: String::new(),
        bps: 16,
        data,
        cpp: 3,
        #[cfg(feature = "native")]
        exif: None,
        id: FrameId::fresh(),
    }
}
