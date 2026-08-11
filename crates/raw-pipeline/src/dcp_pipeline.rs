use std::sync::Arc;

use crate::dcp::DcpProfile;
use crate::edits::Edits;
use crate::frame::RawFrame;
use crate::ops::ResolvedDcp;

pub struct DcpSetup {
    pub cam_to_srgb: [[f32; 3]; 3],
    pub resolved: Option<Arc<ResolvedDcp>>,
}

pub fn resolve(frame: &RawFrame, edits: &Edits, profile: Option<&DcpProfile>) -> DcpSetup {
    let dcp_active = frame.is_raw && edits.color.dcp.is_active() && profile.is_some();
    if let Some(profile) = profile.filter(|_| dcp_active) {
        let (matrix, resolved) =
            crate::ops::resolve_dcp(profile, frame.wb_coeffs, &edits.color.dcp);
        let exposure = if edits.color.dcp.use_baseline_exposure {
            2f32.powf(profile.baseline_exposure_offset)
        } else {
            1.0
        };
        return DcpSetup {
            cam_to_srgb: crate::auto::scale_matrix(matrix, exposure),
            resolved: Some(Arc::new(resolved)),
        };
    }

    let xyz_to_cam =
        crate::color::resolve_xyz_to_cam(&frame.color_matrices, frame.wb_coeffs, frame.xyz_to_cam);
    let cam_to_srgb = if frame.is_raw && !crate::color::is_unusable_matrix(&xyz_to_cam) {
        crate::color::cam_to_srgb_matrix(xyz_to_cam)
    } else {
        crate::color::identity_3x3()
    };
    let resolved = if frame.is_raw && !edits.color.dcp.is_flat() {
        Some(Arc::new(ResolvedDcp::default_color()))
    } else {
        None
    };
    DcpSetup {
        cam_to_srgb,
        resolved,
    }
}
