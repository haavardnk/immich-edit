use std::sync::Arc;

use raw_pipeline::dcp::DcpProfile;
use raw_pipeline::edits::{CropRect, Edits};
use raw_pipeline::frame::{OutputColorSpace, OutputFormat, PreviewMode, RenderOptions};
use raw_pipeline::lut::LutMap;
use raw_pipeline::mask_raster::RasterMap;
use serde::Deserialize;
use wgpu::SurfaceColorSpace;

#[derive(Debug, PartialEq)]
pub struct EditInputs {
    pub sensor_key: String,
    pub rasters: Vec<String>,
    pub lut: Option<String>,
}

impl EditInputs {
    pub fn parse(edits: &str, max_edge: u32) -> serde_json::Result<Self> {
        let edits = serde_json::from_str::<Edits>(edits)?.clamped();
        Ok(Self {
            sensor_key: format!("{}-{max_edge}", edits.sensor_stage().stable_hash()),
            rasters: edits.referenced_raster_ids(),
            lut: edits.referenced_lut_id(),
        })
    }
}

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
    #[serde(default)]
    pub roi: Option<[f32; 4]>,
    #[serde(default)]
    pub tile: bool,
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
            roi: self.roi.map(|[x, y, w, h]| CropRect { x, y, w, h }),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn view(json: &str) -> RenderView {
        serde_json::from_str(json).expect("valid render view")
    }

    fn sensor_key(edits: &str, max_edge: u32) -> String {
        EditInputs::parse(edits, max_edge)
            .expect("valid edits")
            .sensor_key
    }

    #[test]
    fn a_bare_view_renders_srgb_rgb8_without_extras() {
        let opts = view(r#"{"max_edge":1024}"#).options(RasterMap::new(), LutMap::new(), None);
        assert_eq!(opts.max_edge, 1024);
        assert_eq!(opts.output, OutputFormat::Rgb8);
        assert_eq!(opts.output_color_space, OutputColorSpace::SRgb);
        assert_eq!(opts.preview_mode, PreviewMode::None);
        assert!(!opts.gamut_warn && !opts.clip_warn && !opts.histogram && !opts.scopes);
        assert_eq!(opts.roi, None);
        assert!(opts.dcp.is_none());
    }

    #[test]
    fn a_view_without_max_edge_is_rejected() {
        assert!(serde_json::from_str::<RenderView>("{}").is_err());
    }

    #[test]
    fn view_fields_reach_the_render_options() {
        let view = view(
            r#"{"max_edge":512,"output_color_space":"displayp3","gamut_warn":true,"clip_warn":true,
                "preview_mode":{"mask_weight":{"layer_id":"l1"}},"roi":[0.1,0.2,0.3,0.4],"tile":true}"#,
        );
        assert!(view.tile);
        let opts = view.options(RasterMap::new(), LutMap::new(), None);
        assert_eq!(opts.output_color_space, OutputColorSpace::DisplayP3);
        assert!(opts.gamut_warn && opts.clip_warn);
        assert_eq!(
            opts.preview_mode,
            PreviewMode::MaskWeight {
                layer_id: "l1".into()
            }
        );
        assert_eq!(
            opts.roi,
            Some(CropRect {
                x: 0.1,
                y: 0.2,
                w: 0.3,
                h: 0.4
            })
        );
    }

    #[test]
    fn scopes_need_the_histogram() {
        for (histogram, scopes, expected) in [
            (false, false, false),
            (false, true, false),
            (true, false, false),
            (true, true, true),
        ] {
            let json = format!(r#"{{"max_edge":1,"histogram":{histogram},"scopes":{scopes}}}"#);
            let opts = view(&json).options(RasterMap::new(), LutMap::new(), None);
            assert_eq!(opts.histogram, histogram, "{json}");
            assert_eq!(opts.scopes, expected, "{json}");
        }
    }

    #[test]
    fn canvas_color_space_follows_the_output_space() {
        for (space, expected) in [
            ("srgb", SurfaceColorSpace::Srgb),
            ("displayp3", SurfaceColorSpace::DisplayP3),
        ] {
            let json = format!(r#"{{"max_edge":1,"output_color_space":"{space}"}}"#);
            assert_eq!(view(&json).canvas_color_space(), expected, "{space}");
        }
    }

    #[test]
    fn sensor_key_tracks_only_sensor_stage_edits_and_size() {
        let base = sensor_key("{}", 2048);
        assert!(base.ends_with("-2048"), "{base}");
        for (edits, max_edge, same) in [
            (r#"{"basic":{"exposure_ev":2.0}}"#, 2048, true),
            (r#"{"color":{"lut_3d":{"lut_id":"film"}}}"#, 2048, true),
            (r#"{"detail":{"luma_nr_amount":30.0}}"#, 2048, false),
            (r#"{"geometry":{"rotate":90}}"#, 2048, false),
            ("{}", 1024, false),
        ] {
            assert_eq!(
                sensor_key(edits, max_edge) == base,
                same,
                "{edits} @ {max_edge}"
            );
        }
    }

    #[test]
    fn sensor_key_uses_clamped_edits() {
        assert_eq!(
            sensor_key(r#"{"detail":{"luma_nr_amount":500.0}}"#, 1),
            sensor_key(r#"{"detail":{"luma_nr_amount":100.0}}"#, 1)
        );
    }

    #[test]
    fn referenced_rasters_and_lut_are_listed() {
        let brush = |id: &str, raster: &str| {
            format!(r#"{{"id":"{id}","kind":{{"kind":"brush","raster_id":"{raster}"}}}}"#)
        };
        let edits = format!(
            r#"{{"color":{{"lut_3d":{{"lut_id":"film"}}}},
                "masks":[{{"id":"l1","components":[{},{},{}]}}]}}"#,
            brush("a", "r1"),
            brush("b", "r2"),
            brush("c", "r1"),
        );
        let inputs = EditInputs::parse(&edits, 1).expect("valid edits");
        assert_eq!(inputs.rasters, ["r1", "r2"]);
        assert_eq!(inputs.lut.as_deref(), Some("film"));
    }

    #[test]
    fn an_empty_lut_id_is_no_lut() {
        let inputs =
            EditInputs::parse(r#"{"color":{"lut_3d":{"lut_id":""}}}"#, 1).expect("valid edits");
        assert_eq!(inputs.lut, None);
        assert!(inputs.rasters.is_empty());
    }

    #[test]
    fn malformed_edits_are_an_error() {
        assert!(EditInputs::parse(r#"{"basic":{"exposure_ev":"high"}}"#, 1).is_err());
    }
}
