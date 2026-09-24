use half::f16;

use super::{cached_sensor_stage, oriented_frame_dims, prepare};
use crate::cancel::{self, CancelToken};
use crate::cpu::renderer::CpuRenderer;
use crate::edits::Edits;
use crate::frame::{FrameMeta, RawFrame, RenderOptions};
use crate::source::{LinearKind, RenderedSource, SourceHeader, SourceImage};
use crate::timing::{self, StageClock};

pub(crate) fn render_source(
    frame: &RawFrame,
    edits: &Edits,
    options: &RenderOptions,
    cancel: Option<&CancelToken>,
    renderer: &CpuRenderer,
) -> crate::PipelineResult<RenderedSource> {
    let prep = prepare(frame, edits, options);
    let clock = StageClock::default();
    let stage = cached_sensor_stage(frame, &prep, options, renderer, &clock, cancel)?;
    cancel::check(cancel)?;
    let atmosphere = clock.time(timing::DEHAZE, || {
        crate::cpu::dehaze::atmosphere_for_render(&stage.rgb, stage.width, stage.height)
    });
    let (width, height) = oriented_frame_dims(frame);
    let meta = FrameMeta {
        width,
        height,
        orientation: (false, false, false),
        ..frame.meta.clone()
    };
    let rgb_f16 = stage
        .rgb
        .iter()
        .map(|v| f16::from_f32(*v).to_bits())
        .collect();
    Ok(RenderedSource {
        image: SourceImage {
            header: SourceHeader {
                meta,
                kind: LinearKind::PostWb,
                dims: (stage.width as u32, stage.height as u32),
                atmosphere: Some(atmosphere),
            },
            rgb_f16,
        },
        renderer: "cpu".into(),
        timings: clock.finish(),
    })
}
