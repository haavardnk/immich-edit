pub type OrientFlips = (bool, bool, bool);

pub struct RawFrame {
    pub width: usize,
    pub height: usize,
    pub cfa_pattern: String,
    pub bps: usize,
    pub wb_coeffs: [f32; 4],
    pub xyz_to_cam: [[f32; 3]; 4],
    pub color_matrices: Vec<(f32, [[f32; 3]; 4])>,
    pub black_levels: [f32; 4],
    pub white_levels: [f32; 4],
    pub data: Vec<f32>,
    pub cpp: usize,
    pub orientation: OrientFlips,
    pub is_raw: bool,
    pub exif: Option<little_exif::metadata::Metadata>,
}

pub struct RenderOptions {
    pub max_edge: u32,
    pub quality: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            max_edge: 4096,
            quality: false,
        }
    }
}

pub struct RenderedImage {
    pub jpeg: Vec<u8>,
    pub histogram: crate::histogram::Histogram,
    pub linear_histogram: Option<crate::histogram::Histogram>,
    pub width: u32,
    pub height: u32,
    pub renderer: String,
}
