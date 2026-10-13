mod measure;
mod proxy;
mod scene;
mod solve;
mod weights;

#[cfg(test)]
mod tests;

use std::sync::Arc;

use crate::PipelineResult;
use crate::dcp::DcpProfile;
use crate::edits::Edits;
use crate::frame::RawFrame;
use crate::white_balance::auto_white_balance;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Face {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub fn auto_adjust(
    frame: &RawFrame,
    context: &Edits,
    profile: Option<Arc<DcpProfile>>,
    faces: &[Face],
) -> PipelineResult<Edits> {
    let context = context.clamped();
    let mut base = proxy::metering_edits(&context);
    if !base.color.bw.enabled
        && let Some(white_balance) = auto_white_balance(frame, &base, profile.as_deref())
    {
        base.basic.white_balance = white_balance;
    }
    let white_balance = base.basic.white_balance;
    let mut proxy = proxy::Proxy::new(frame, base, profile, faces)?;
    let sliders = solve::solve(&mut proxy, scene::exposure_value(frame))?;
    let mut edits = context;
    sliders.write(&mut edits, proxy.colour());
    edits.basic.white_balance = white_balance;
    Ok(edits)
}
