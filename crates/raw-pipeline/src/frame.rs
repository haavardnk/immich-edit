use crate::edits::Edits;

pub type OrientFlips = (bool, bool, bool);

pub struct RawFrame {
    pub width: usize,
    pub height: usize,
    pub cfa_pattern: String,
    pub bps: usize,
    pub wb_coeffs: [f32; 4],
    pub cam_to_xyz: [[f32; 4]; 3],
    pub black_levels: [f32; 4],
    pub white_levels: [f32; 4],
    pub data: Vec<f32>,
    pub cpp: usize,
    pub orientation: OrientFlips,
}

impl RawFrame {
    pub fn byte_size(&self) -> usize {
        self.data.len() * std::mem::size_of::<f32>()
            + self.cfa_pattern.len()
            + std::mem::size_of::<Self>()
    }
}

pub struct RenderOptions {
    pub max_edge: u32,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self { max_edge: 2048 }
    }
}

pub struct RenderedImage {
    pub jpeg: Vec<u8>,
    pub histogram: crate::histogram::Histogram,
    pub width: u32,
    pub height: u32,
    pub renderer: String,
}

pub trait Renderer: Send + Sync {
    fn render(
        &self,
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
    ) -> crate::PipelineResult<RenderedImage>;

    fn name(&self) -> &str;
}
