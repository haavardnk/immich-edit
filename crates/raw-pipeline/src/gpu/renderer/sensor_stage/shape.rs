use wgpu::Limits;

use super::super::upload::sensor_dims;
use crate::edits::Edits;
use crate::frame::{RawFrame, RenderOptions};
use crate::geom;
use crate::ops::capture_sharpen::{CaptureSigma, frame_sigma};
use crate::tile::{Rect, SensorChain, SensorTile, Warp, support};

const MOSAIC_TEXEL: u64 = 4;

pub(in crate::gpu::renderer) struct SensorShape {
    pub block: Option<usize>,
    pub sigma: Option<CaptureSigma>,
    pub chain: SensorChain,
}

impl SensorShape {
    pub fn new(frame: &RawFrame, edits: &Edits, options: &RenderOptions, presence: bool) -> Self {
        let full = (frame.meta.width as u32, frame.meta.height as u32);
        let ratio = geom::preview_ratio(
            frame.meta.orientation,
            edits,
            full,
            options.max_edge,
            options.quality,
        );
        let block = geom::superpixel_block(frame, options, ratio);
        let scale = block.unwrap_or(1);
        let sensor = sensor_dims(frame, scale);
        let sigma =
            frame_sigma(frame, edits, scale).filter(|_| presence && sensor.0 >= 8 && sensor.1 >= 8);
        let working = ratio
            .filter(|_| presence)
            .and_then(|ratio| geom::resample_target(sensor, ratio / scale as f32))
            .unwrap_or(sensor);
        let detail = if presence {
            let d = &edits.detail;
            let capture = sigma.map_or(0, |sigma| {
                let radius = sigma.kernels().iter().map(Vec::len).max().unwrap_or(1) / 2;
                support::capture_sharpen(radius as u32)
            });
            support::noise_reduction(d.luma_nr_active(), d.color_nr_active()) + capture
        } else {
            0
        };
        let warps = frame
            .embedded_lens
            .warp()
            .cloned()
            .map(Warp::Dng)
            .into_iter()
            .chain(edits.lens.any_active().then_some(Warp::Lens(edits.lens)))
            .collect();
        Self {
            block,
            sigma,
            chain: SensorChain {
                sensor,
                working,
                align: support::alignment(&frame.cfa_pattern, frame.cpp, scale),
                demosaic: support::demosaic(&frame.cfa_pattern, frame.cpp, scale),
                warps,
                detail,
            },
        }
    }

    pub fn sensor(&self) -> (u32, u32) {
        self.chain.sensor
    }

    pub fn working(&self) -> (u32, u32) {
        self.chain.working
    }

    pub fn resamples(&self) -> bool {
        self.chain.working != self.chain.sensor
    }

    pub fn mosaic_window(&self, frame: &RawFrame, rect: Rect) -> Rect {
        if rect == Rect::full(self.chain.sensor) {
            return Rect::full((frame.meta.width as u32, frame.meta.height as u32));
        }
        rect.scaled(self.block.unwrap_or(1) as u32)
    }

    pub fn fits(&self, frame: &RawFrame, tile: &SensorTile, limits: &Limits) -> bool {
        let texture = limits.max_texture_dimension_2d;
        let storage = limits
            .max_storage_buffer_binding_size
            .min(limits.max_buffer_size);
        let window = self.mosaic_window(frame, tile.input());
        let mosaic = u64::from(window.w) * u64::from(window.h) * frame.cpp as u64 * MOSAIC_TEXEL;
        let uploads_rgb = frame.cpp == 3 && self.block.is_none();
        tile.rects.iter().all(|rect| rect.w.max(rect.h) <= texture)
            && (uploads_rgb || mosaic <= storage)
    }
}
