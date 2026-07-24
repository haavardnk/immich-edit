use std::sync::Arc;

use super::{FusedOp, OpContext, OpMeta, Stage};
use crate::cpu::fused::CpuFusedOp;
use crate::edits::Edits;

pub struct DcpHueSatOp;

impl OpMeta for DcpHueSatOp {
    fn id(&self) -> &'static str {
        "dcp_hue_sat"
    }
    fn stage(&self) -> Stage {
        Stage::Color
    }
    fn order(&self) -> i32 {
        190
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.color.dcp.is_active() && edits.color.dcp.use_base_table
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        let dcp = &edits.color.dcp;
        if !dcp.is_active() {
            return None;
        }
        serde_json::to_value(dcp).ok()
    }
    fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        if let Ok(dcp) = serde_json::from_value::<crate::edits::DcpEdits>(value.clone()) {
            edits.color.dcp = dcp;
        }
    }
}

impl FusedOp for DcpHueSatOp {
    fn cpu_fused(&self, _edits: &Edits, ctx: &OpContext) -> Option<CpuFusedOp> {
        let dcp = ctx.render.dcp.as_ref()?;
        let map = dcp.base_table.as_ref()?;
        Some(CpuFusedOp::DcpHueSat {
            map: Arc::new(map.clone()),
            to_pp: dcp.to_pp,
            from_pp: dcp.from_pp,
        })
    }
}
