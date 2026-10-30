use crate::edits::Edits;
use crate::frame::FrameMeta;
use crate::geom;
#[cfg(feature = "native")]
use crate::tile::DisplayTile;
use crate::tile::Rect;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::gpu::renderer) struct OutputShape {
    pub full: (u32, u32),
    pub rect: Rect,
    pub core: Rect,
    pub scale: f32,
    pub ratio: f32,
}

impl OutputShape {
    pub fn new(meta: &FrameMeta, edits: &Edits, max_edge: u32, working: (u32, u32)) -> Self {
        let dims = (meta.width as u32, meta.height as u32);
        let full = geom::display_out_dims(meta.orientation, edits, dims, max_edge);
        let (crop_w, crop_h) = geom::display_crop_px(meta.orientation, edits, working);
        Self {
            full,
            rect: Rect::full(full),
            core: Rect::full(full),
            scale: geom::output_scale(meta.orientation, edits, dims, full),
            ratio: (crop_w as f32 / full.0 as f32).max(crop_h as f32 / full.1 as f32),
        }
    }

    #[cfg(feature = "native")]
    pub fn tile(self, tile: &DisplayTile) -> Self {
        Self {
            rect: tile.rect,
            core: tile.core,
            ..self
        }
    }

    pub fn dims(&self) -> (u32, u32) {
        self.rect.dims()
    }

    pub fn origin(&self) -> (u32, u32) {
        (self.rect.x, self.rect.y)
    }

    pub fn output(&self) -> [i32; 4] {
        [
            self.rect.x as i32,
            self.rect.y as i32,
            self.rect.w as i32,
            self.rect.h as i32,
        ]
    }

    pub fn target(&self, working: (u32, u32)) -> Option<(u32, u32)> {
        geom::resample_target(working, self.ratio)
    }
}
