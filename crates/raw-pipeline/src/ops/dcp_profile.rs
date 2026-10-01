use super::{GpuOp, Op, OpContext, ResolvedDcp, Stage};
use crate::auto::scale_matrix;
use crate::cpu::fused::CpuFusedOp;
use crate::edits::Edits;

pub const DCP_PROFILE_OP_ID: &str = "dcp_hue_sat";

pub struct DcpProfileOp;

fn base_matrices(dcp: &ResolvedDcp) -> ([[f32; 3]; 3], [[f32; 3]; 3]) {
    (
        scale_matrix(dcp.to_pp, 1.0 / dcp.baseline_gain),
        scale_matrix(dcp.from_pp, dcp.baseline_gain),
    )
}

impl Op for DcpProfileOp {
    fn id(&self) -> &'static str {
        DCP_PROFILE_OP_ID
    }
    fn gpu_route(&self) -> super::GpuRoute {
        super::GpuRoute::Fused
    }
    fn stage(&self) -> Stage {
        Stage::Tone
    }
    fn order(&self) -> i32 {
        -5
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.color.dcp.is_active() && edits.color.dcp.use_base_table
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        let dcp = &edits.color.dcp;
        if dcp == &crate::edits::DcpEdits::default() {
            return None;
        }
        serde_json::to_value(dcp).ok()
    }
    fn apply_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        if let Ok(dcp) = serde_json::from_value::<crate::edits::DcpEdits>(value.clone()) {
            edits.color.dcp = dcp;
        }
    }
    fn gpu(&self) -> Option<GpuOp> {
        Some(GpuOp {
            field_name: "dcp_base",
            functions: concat!(
                include_str!("../../assets/shaders/huesat_table.wgsl"),
                include_str!("../../assets/shaders/ops/dcp_base.wgsl"),
            ),
            apply: "lin = dcp_base_apply(lin);",
            vec4_count: 7,
        })
    }
    fn cpu_fused(&self, _edits: &Edits, ctx: &OpContext) -> Option<CpuFusedOp> {
        let dcp = ctx.render.dcp.as_ref()?;
        let map = dcp.base_table.as_ref()?;
        let (to_pp, from_pp) = base_matrices(dcp);
        Some(CpuFusedOp::DcpHueSat {
            map: map.clone(),
            to_pp,
            from_pp,
        })
    }
    fn write_gpu_uniform(&self, _edits: &Edits, ctx: &OpContext, dst: &mut [f32]) {
        let Some(dcp) = ctx.render.dcp.as_ref() else {
            return;
        };
        let Some(map) = dcp.base_table.as_ref() else {
            return;
        };
        let (to_pp, from_pp) = base_matrices(dcp);
        for (i, row) in to_pp.iter().chain(from_pp.iter()).enumerate() {
            dst[i * 4..i * 4 + 3].copy_from_slice(row);
        }
        for (slot, dim) in dst[24..28].iter_mut().zip(map.gpu_dims()) {
            *slot = dim as f32;
        }
    }
}
