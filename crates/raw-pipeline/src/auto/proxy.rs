use std::sync::Arc;

use super::Face;
use super::measure::{Image, Metrics};
use super::solve::Sliders;
use super::weights::Weights;
use crate::cpu::CpuRenderer;
use crate::dcp::DcpProfile;
use crate::edits::{BasicEdits, ColorEdits, Edits};
use crate::frame::{OutputFormat, RawFrame, RenderOptions};
use crate::{PipelineError, PipelineResult};

const EDGE: u32 = 320;
const CLIP_LEVEL: f32 = 0.985;

pub(super) struct Proxy<'a> {
    frame: &'a RawFrame,
    renderer: CpuRenderer,
    options: RenderOptions,
    base: Edits,
    colour: bool,
    weights: Weights,
    last: (Sliders, Image),
}

pub(super) fn metering_edits(context: &Edits) -> Edits {
    let colour = !context.color.bw.enabled;
    Edits {
        basic: BasicEdits {
            white_balance: context.basic.white_balance,
            texture: context.basic.texture,
            saturation: if colour {
                0.0
            } else {
                context.basic.saturation
            },
            vibrance: if colour { 0.0 } else { context.basic.vibrance },
            ..Default::default()
        },
        color: ColorEdits {
            dcp: context.color.dcp.clone(),
            bw: context.color.bw,
            ..Default::default()
        },
        lens: context.lens,
        geometry: context.geometry.clone(),
        retouch: context.retouch.clone(),
        ..Default::default()
    }
}

impl<'a> Proxy<'a> {
    pub(super) fn new(
        frame: &'a RawFrame,
        base: Edits,
        profile: Option<Arc<DcpProfile>>,
        faces: &[Face],
    ) -> PipelineResult<Self> {
        let renderer = CpuRenderer::new();
        let options = RenderOptions {
            max_edge: EDGE,
            output: OutputFormat::Rgb8,
            dcp: profile,
            ..RenderOptions::default()
        };
        let colour = !base.color.bw.enabled;
        let image = render(&renderer, frame, &base, &options)?;
        let weights = Weights::new(frame, &base, faces, image.width, image.height);
        Ok(Self {
            frame,
            renderer,
            options,
            base,
            colour,
            weights,
            last: (Sliders::default(), image),
        })
    }

    pub(super) fn colour(&self) -> bool {
        self.colour
    }

    pub(super) fn face_mass(&self) -> f64 {
        self.weights.face_mass
    }

    pub(super) fn metrics(&mut self, sliders: &Sliders) -> PipelineResult<Metrics> {
        let (image, weights) = self.view(sliders)?;
        Ok(image.metrics(weights))
    }

    pub(super) fn local_contrast(&mut self, sliders: &Sliders) -> PipelineResult<f64> {
        let (image, weights) = self.view(sliders)?;
        Ok(image.local_contrast(weights))
    }

    pub(super) fn haze(&mut self, sliders: &Sliders) -> PipelineResult<f64> {
        let (image, weights) = self.view(sliders)?;
        Ok(image.haze(weights))
    }

    pub(super) fn clipped_share(&mut self, sliders: &Sliders) -> PipelineResult<f64> {
        let recovered = Sliders {
            highlights: -100.0,
            ..*sliders
        };
        let (image, _) = self.view(&recovered)?;
        Ok(image.clipped_share(CLIP_LEVEL))
    }

    pub(super) fn exclude_clipped(&mut self, sliders: &Sliders) -> PipelineResult<()> {
        let recovered = Sliders {
            highlights: -100.0,
            whites: -100.0,
            ..*sliders
        };
        let (image, _) = self.view(&recovered)?;
        let clipped = image.clipped(CLIP_LEVEL);
        self.weights.exclude(&clipped);
        Ok(())
    }

    fn view(&mut self, sliders: &Sliders) -> PipelineResult<(&Image, &Weights)> {
        if self.last.0 != *sliders {
            let edits = sliders.edits(&self.base, self.colour);
            let image = render(&self.renderer, self.frame, &edits, &self.options)?;
            self.last = (*sliders, image);
        }
        Ok((&self.last.1, &self.weights))
    }
}

fn render(
    renderer: &CpuRenderer,
    frame: &RawFrame,
    edits: &Edits,
    options: &RenderOptions,
) -> PipelineResult<Image> {
    let rendered = renderer.render(frame, edits, options)?;
    let width = rendered.width as usize;
    let height = rendered.height as usize;
    if width == 0 || height == 0 || rendered.bytes.len() < width * height * 3 {
        return Err(PipelineError::Render("auto proxy render is empty".into()));
    }
    Ok(Image::from_rgb8(&rendered.bytes, width, height))
}
