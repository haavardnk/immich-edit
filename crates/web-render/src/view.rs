use std::sync::Arc;

use raw_pipeline::dcp::DcpProfile;
use raw_pipeline::frame::{OutputColorSpace, OutputFormat, PreviewMode, RenderOptions};
use raw_pipeline::lut::LutMap;
use raw_pipeline::mask_raster::RasterMap;
use serde::Deserialize;
use wgpu::SurfaceColorSpace;

#[derive(Debug, Deserialize)]
pub struct RenderView {
    pub max_edge: u32,
    #[serde(default)]
    pub output_color_space: OutputColorSpace,
    #[serde(default)]
    pub preview_mode: PreviewMode,
    #[serde(default)]
    pub gamut_warn: bool,
    #[serde(default)]
    pub clip_warn: bool,
    #[serde(default)]
    pub histogram: bool,
    #[serde(default)]
    pub scopes: bool,
}

impl RenderView {
    pub fn options(
        &self,
        rasters: RasterMap,
        luts: LutMap,
        dcp: Option<Arc<DcpProfile>>,
    ) -> RenderOptions {
        RenderOptions {
            max_edge: self.max_edge,
            output: OutputFormat::Rgb8,
            output_color_space: self.output_color_space,
            preview_mode: self.preview_mode.clone(),
            gamut_warn: self.gamut_warn,
            clip_warn: self.clip_warn,
            histogram: self.histogram,
            scopes: self.histogram && self.scopes,
            rasters,
            luts,
            dcp,
            ..Default::default()
        }
    }

    pub fn canvas_color_space(&self) -> SurfaceColorSpace {
        match self.output_color_space {
            OutputColorSpace::SRgb => SurfaceColorSpace::Srgb,
            OutputColorSpace::DisplayP3 => SurfaceColorSpace::DisplayP3,
        }
    }
}
