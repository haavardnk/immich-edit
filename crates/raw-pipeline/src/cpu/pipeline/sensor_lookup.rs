use std::sync::Arc;

use super::ops::{OpRange, run_pipeline_ops_inner};
use super::{Prepared, oriented_sensor};
use crate::cancel::CancelToken;
use crate::cpu::renderer::{CpuRenderer, SensorStage, sensor_cache_key};
use crate::frame::{RawFrame, RenderOptions};
use crate::timing::StageClock;

pub(super) struct SensorLookup<'a> {
    renderer: &'a CpuRenderer,
    frame: &'a RawFrame,
    prep: &'a Prepared,
    options: &'a RenderOptions,
    key: u64,
    hit: Option<Arc<SensorStage>>,
}

impl<'a> SensorLookup<'a> {
    pub fn new(
        renderer: &'a CpuRenderer,
        frame: &'a RawFrame,
        prep: &'a Prepared,
        options: &'a RenderOptions,
    ) -> Self {
        let key = sensor_cache_key(
            frame,
            &prep.edits,
            &prep.setup,
            options,
            prep.preview_ratio,
            prep.block,
            prep.denoise.as_ref().map(|(plane, _)| plane.as_ref()),
        );
        Self {
            renderer,
            frame,
            prep,
            options,
            key,
            hit: renderer.get(key),
        }
    }

    pub fn dims(&self) -> Option<(usize, usize)> {
        self.hit.as_ref().map(|stage| (stage.width, stage.height))
    }

    pub fn stage(
        self,
        clock: &StageClock,
        cancel: Option<&CancelToken>,
    ) -> crate::PipelineResult<Arc<SensorStage>> {
        if let Some(stage) = self.hit {
            return Ok(stage);
        }
        let prep = self.prep;
        let (mut image, oriented, preview_dims) = oriented_sensor(self.frame, prep, clock, cancel)?;
        run_pipeline_ops_inner(
            &mut image,
            prep.op_inputs(self.options),
            OpRange::BelowBoundary,
            preview_dims,
            clock,
            cancel,
        )?;
        let stage = Arc::new(SensorStage {
            rgb: image.rgb,
            width: image.width,
            height: image.height,
            oriented_w: oriented.0,
            oriented_h: oriented.1,
        });
        self.renderer.put(self.key, stage.clone());
        Ok(stage)
    }
}
