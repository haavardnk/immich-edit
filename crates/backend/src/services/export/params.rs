use raw_pipeline::frame::{
    BitDepth, JpegSubsampling, OutputColorSpace, OutputFormat, PngCompression, TiffCompression,
};
use serde::Deserialize;

use super::DEFAULT_QUALITY;

fn default_quality() -> u8 {
    DEFAULT_QUALITY
}

fn default_include_exif() -> bool {
    true
}

pub fn default_suffix() -> String {
    "_edit".into()
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
        }
    }
}

impl ExportParams {
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
