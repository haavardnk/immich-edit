pub type OrientFlips = (bool, bool, bool);

pub struct RawFrame {
    pub width: usize,
    pub height: usize,
    pub cfa_pattern: String,
    pub bps: usize,
    pub wb_coeffs: [f32; 4],
    pub xyz_to_cam: [[f32; 3]; 4],
    pub color_matrices: Vec<(f32, [[f32; 3]; 4])>,
    pub data: Vec<f32>,
    pub cpp: usize,
    pub orientation: OrientFlips,
    pub is_raw: bool,
    pub exif: Option<little_exif::metadata::Metadata>,
}

pub struct RenderOptions {
    pub max_edge: u32,
    pub quality: bool,
    pub output: OutputFormat,
    pub preview_mode: PreviewMode,
    pub rasters: crate::mask_raster::RasterMap,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreviewMode {
    #[default]
    None,
    SharpenMask,
    SharpenRadius,
    SharpenDetail,
    MaskWeight {
        layer_id: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitDepth {
    Eight,
    Sixteen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PngCompression {
    Fast,
    Default,
    Best,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TiffCompression {
    None,
    Lzw,
    Deflate,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Jpeg {
        quality: u8,
    },
    Png {
        bit_depth: BitDepth,
        compression: PngCompression,
    },
    Webp {
        quality: u8,
        lossless: bool,
    },
    Avif {
        quality: u8,
    },
    Heic {
        quality: u8,
    },
    Tiff {
        bit_depth: BitDepth,
        compression: TiffCompression,
    },
    Jxl {
        bit_depth: BitDepth,
    },
}

impl OutputFormat {
    pub fn content_type(&self) -> &'static str {
        match self {
            Self::Jpeg { .. } => "image/jpeg",
            Self::Png { .. } => "image/png",
            Self::Webp { .. } => "image/webp",
            Self::Avif { .. } => "image/avif",
            Self::Heic { .. } => "image/heic",
            Self::Tiff { .. } => "image/tiff",
            Self::Jxl { .. } => "image/jxl",
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Jpeg { .. } => "jpg",
            Self::Png { .. } => "png",
            Self::Webp { .. } => "webp",
            Self::Avif { .. } => "avif",
            Self::Heic { .. } => "heic",
            Self::Tiff { .. } => "tif",
            Self::Jxl { .. } => "jxl",
        }
    }

    pub fn bit_depth(&self) -> BitDepth {
        match self {
            Self::Jpeg { .. } | Self::Webp { .. } | Self::Avif { .. } | Self::Heic { .. } => {
                BitDepth::Eight
            }
            Self::Png { bit_depth, .. }
            | Self::Tiff { bit_depth, .. }
            | Self::Jxl { bit_depth } => *bit_depth,
        }
    }

    pub fn exif_file_extension(&self) -> little_exif::filetype::FileExtension {
        use little_exif::filetype::FileExtension;
        match self {
            Self::Jpeg { .. } => FileExtension::JPEG,
            Self::Png { .. } => FileExtension::PNG {
                as_zTXt_chunk: true,
            },
            Self::Webp { .. } => FileExtension::WEBP,
            Self::Avif { .. } => FileExtension::HEIF,
            Self::Heic { .. } => FileExtension::HEIF,
            Self::Tiff { .. } => FileExtension::TIFF,
            Self::Jxl { .. } => FileExtension::JXL,
        }
    }
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            max_edge: 4096,
            quality: false,
            output: OutputFormat::Jpeg { quality: 85 },
            preview_mode: PreviewMode::None,
            rasters: crate::mask_raster::empty_rasters(),
        }
    }
}

pub struct RenderedImage {
    pub bytes: Vec<u8>,
    pub histogram: crate::histogram::Histogram,
    pub linear_histogram: Option<crate::histogram::Histogram>,
    pub width: u32,
    pub height: u32,
    pub source_w: u32,
    pub source_h: u32,
    pub renderer: String,
}
