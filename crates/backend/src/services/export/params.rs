use raw_pipeline::frame::{
    BitDepth, JpegSubsampling, OutputColorSpace, OutputFormat, OutputSharpen, PngCompression,
    TiffCompression,
};
use serde::Deserialize;

use super::DEFAULT_QUALITY;
use super::output_sharpen::{SharpenAmountOpt, SharpenMediaOpt, output_sharpen};
use super::resize::{Resize, ResizeMode};
use super::watermark::{
    DEFAULT_INSET, DEFAULT_OPACITY, DEFAULT_SIZE, WatermarkAnchorOpt, WatermarkPlacement,
    watermark_placement,
};
use crate::error::AppError;

fn default_quality() -> u8 {
    DEFAULT_QUALITY
}

fn default_watermark_size() -> f32 {
    DEFAULT_SIZE
}

fn default_watermark_opacity() -> f32 {
    DEFAULT_OPACITY
}

fn default_watermark_inset() -> f32 {
    DEFAULT_INSET
}

fn default_include_exif() -> bool {
    true
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormatKind {
    #[default]
    Jpeg,
    Png,
    Webp,
    Avif,
    Heic,
    Tiff,
    Jxl,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum BitDepthOpt {
    #[default]
    #[serde(rename = "8")]
    Eight,
    #[serde(rename = "16")]
    Sixteen,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PngCompressionOpt {
    Fast,
    #[default]
    Default,
    Best,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TiffCompressionOpt {
    None,
    #[default]
    Lzw,
    Deflate,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ColorSpaceOpt {
    #[default]
    Srgb,
    Displayp3,
}

#[derive(Debug, Deserialize)]
pub struct ExportParams {
    #[serde(default)]
    pub format: ExportFormatKind,
    #[serde(default = "default_quality")]
    pub quality: u8,
    #[serde(default = "default_include_exif")]
    pub include_exif: bool,
    #[serde(default)]
    pub bit_depth: BitDepthOpt,
    #[serde(default)]
    pub png_compression: PngCompressionOpt,
    #[serde(default)]
    pub tiff_compression: TiffCompressionOpt,
    #[serde(default)]
    pub lossless: bool,
    #[serde(default)]
    pub color_space: ColorSpaceOpt,
    #[serde(default)]
    pub filename_template: Option<String>,
    #[serde(default)]
    pub resize_mode: Option<ResizeMode>,
    #[serde(default)]
    pub resize_width: Option<u32>,
    #[serde(default)]
    pub resize_height: Option<u32>,
    #[serde(default)]
    pub resize_megapixels: Option<f64>,
    #[serde(default)]
    pub resize_percent: Option<f64>,
    #[serde(default)]
    pub resize_enlarge: bool,
    #[serde(default)]
    pub output_sharpen_media: Option<SharpenMediaOpt>,
    #[serde(default)]
    pub output_sharpen_amount: SharpenAmountOpt,
    #[serde(default)]
    pub output_sharpen_ppi: Option<u32>,
    #[serde(default)]
    pub watermark_id: Option<String>,
    #[serde(default = "default_watermark_size")]
    pub watermark_size: f32,
    #[serde(default = "default_watermark_opacity")]
    pub watermark_opacity: f32,
    #[serde(default)]
    pub watermark_anchor: WatermarkAnchorOpt,
    #[serde(default = "default_watermark_inset")]
    pub watermark_inset: f32,
}

impl Default for ExportParams {
    fn default() -> Self {
        Self {
            format: ExportFormatKind::default(),
            quality: DEFAULT_QUALITY,
            include_exif: true,
            bit_depth: BitDepthOpt::default(),
            png_compression: PngCompressionOpt::default(),
            tiff_compression: TiffCompressionOpt::default(),
            lossless: false,
            color_space: ColorSpaceOpt::default(),
            filename_template: None,
            resize_mode: None,
            resize_width: None,
            resize_height: None,
            resize_megapixels: None,
            resize_percent: None,
            resize_enlarge: false,
            output_sharpen_media: None,
            output_sharpen_amount: SharpenAmountOpt::default(),
            output_sharpen_ppi: None,
            watermark_id: None,
            watermark_size: DEFAULT_SIZE,
            watermark_opacity: DEFAULT_OPACITY,
            watermark_anchor: WatermarkAnchorOpt::default(),
            watermark_inset: DEFAULT_INSET,
        }
    }
}

impl ExportParams {
    pub fn resize(&self) -> Result<Option<Resize>, AppError> {
        self.resize_mode
            .map(|mode| match mode {
                ResizeMode::Dimensions => {
                    Resize::fit(self.resize_width, self.resize_height, self.resize_enlarge)
                }
                ResizeMode::Megapixels => {
                    Resize::megapixels(self.resize_megapixels, self.resize_enlarge)
                }
                ResizeMode::Percent => Resize::percent(self.resize_percent),
            })
            .transpose()
    }

    pub fn output_sharpen(&self) -> Result<Option<OutputSharpen>, AppError> {
        output_sharpen(
            self.output_sharpen_media,
            self.output_sharpen_amount,
            self.output_sharpen_ppi,
        )
    }

    pub fn watermark(&self) -> Result<Option<WatermarkPlacement>, AppError> {
        watermark_placement(
            self.watermark_id.as_deref(),
            self.watermark_size,
            self.watermark_opacity,
            self.watermark_anchor,
            self.watermark_inset,
        )
    }

    pub fn output_color_space(&self) -> OutputColorSpace {
        match self.color_space {
            ColorSpaceOpt::Srgb => OutputColorSpace::SRgb,
            ColorSpaceOpt::Displayp3 => OutputColorSpace::DisplayP3,
        }
    }

    pub fn output_format(&self) -> OutputFormat {
        let quality = self.quality.clamp(1, 100);
        let bd = match self.bit_depth {
            BitDepthOpt::Eight => BitDepth::Eight,
            BitDepthOpt::Sixteen => BitDepth::Sixteen,
        };
        let png_c = match self.png_compression {
            PngCompressionOpt::Fast => PngCompression::Fast,
            PngCompressionOpt::Default => PngCompression::Default,
            PngCompressionOpt::Best => PngCompression::Best,
        };
        let tiff_c = match self.tiff_compression {
            TiffCompressionOpt::None => TiffCompression::None,
            TiffCompressionOpt::Lzw => TiffCompression::Lzw,
            TiffCompressionOpt::Deflate => TiffCompression::Deflate,
        };
        match self.format {
            ExportFormatKind::Jpeg => OutputFormat::Jpeg {
                quality,
                subsampling: JpegSubsampling::Chroma420,
            },
            ExportFormatKind::Png => OutputFormat::Png {
                bit_depth: bd,
                compression: png_c,
            },
            ExportFormatKind::Webp => OutputFormat::Webp {
                quality,
                lossless: self.lossless || self.include_exif,
            },
            ExportFormatKind::Avif => OutputFormat::Avif { quality },
            ExportFormatKind::Heic => OutputFormat::Heic { quality },
            ExportFormatKind::Tiff => OutputFormat::Tiff {
                bit_depth: bd,
                compression: tiff_c,
            },
            ExportFormatKind::Jxl => OutputFormat::Jxl { bit_depth: bd },
        }
    }
}
