use half::f16;

use super::{cached_sensor_stage, oriented_frame_dims, prepare};
use crate::cancel::{self, CancelToken};
use crate::cpu::renderer::CpuRenderer;
use crate::edits::Edits;
use crate::frame::{FrameMeta, RawFrame, RenderOptions};
use crate::source::{LinearKind, RenderedSource, SourceHeader, SourceImage, SourceWindow};
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
    let full = (stage.width as u32, stage.height as u32);
    let window = options
        .roi
        .and_then(|_| crate::source::window_rect(&meta, &prep.edits, full));
    let to_f16 = |v: &f32| f16::from_f32(*v).to_bits();
    let rgb_f16 = match window {
        Some(rect) => crate::source::crop_rgb(&stage.rgb, full.0, rect)
            .iter()
            .map(to_f16)
            .collect(),
        None => stage.rgb.iter().map(to_f16).collect(),
    };
    Ok(RenderedSource {
        image: SourceImage {
            header: SourceHeader {
                meta,
                kind: LinearKind::PostWb,
                dims: window.map_or(full, |rect| rect.dims),
                atmosphere: Some(atmosphere),
                window: window.map(|rect| SourceWindow {
                    origin: rect.origin,
                    full,
                }),
            },
            rgb_f16,
        },
        renderer: "cpu".into(),
        timings: clock.finish(),
    })
}
