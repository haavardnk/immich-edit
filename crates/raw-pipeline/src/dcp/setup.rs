use std::sync::Arc;

use crate::color::{CameraWhite, SceneWhite};
use crate::dcp::DcpProfile;
use crate::edits::Edits;
use crate::frame::FrameMeta;
use crate::ops::ResolvedDcp;

pub struct DcpSetup {
    pub cam_to_srgb: [[f32; 3]; 3],
    pub resolved: Option<Arc<ResolvedDcp>>,
    pub white: SceneWhite,
}

pub fn resolve(meta: &FrameMeta, edits: &Edits, profile: Option<&DcpProfile>) -> DcpSetup {
    let dcp_active = meta.is_raw && edits.color.dcp.is_active() && profile.is_some();
    if let Some(profile) = profile.filter(|_| dcp_active) {
        let camera = CameraWhite::from_dcp(profile, edits.color.dcp.illuminant, meta.wb_coeffs);
        let weight = camera.weight_at(edits.basic.wb_temp, edits.basic.wb_tint);
        let resolved = crate::ops::resolve_dcp(profile, &edits.color.dcp, weight);
        return DcpSetup {
            cam_to_srgb: crate::auto::scale_matrix(camera.cam_to_srgb(), resolved.baseline_gain),
            resolved: Some(Arc::new(resolved)),
            white: SceneWhite::Camera(camera),
        };
    }

    let white = matrix_white(meta);
    let resolved = if meta.is_raw && !edits.color.dcp.is_flat() {
        Some(Arc::new(ResolvedDcp::default_color()))
    } else {
        None
    };
    DcpSetup {
        cam_to_srgb: white.cam_to_srgb(),
        resolved,
        white,
    }
}

pub(crate) fn matrix_white(meta: &FrameMeta) -> SceneWhite {
    if !meta.is_raw {
        return SceneWhite::Display;
    }
    CameraWhite::from_matrices(&meta.color_matrices, meta.xyz_to_cam, meta.wb_coeffs)
        .map_or(SceneWhite::Display, SceneWhite::Camera)
}
