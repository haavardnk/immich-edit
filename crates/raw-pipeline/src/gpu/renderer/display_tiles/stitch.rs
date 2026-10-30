use crate::gpu::renderer::meta::MetaCounts;
use crate::gpu::renderer::output::{DisplayBuf, ReadbackImage};
use crate::gpu::timer::RenderTimings;
use crate::tile::{DisplayTile, Rect};
use crate::{PipelineError, PipelineResult};

pub(super) struct Stitch {
    full: (u32, u32),
    display: Option<DisplayBuf>,
    counts: MetaCounts,
    source_dims: (u32, u32),
}

impl Stitch {
    pub fn new(full: (u32, u32)) -> Self {
        Self {
            full,
            display: None,
            counts: MetaCounts::default(),
            source_dims: full,
        }
    }

    pub fn place<'a>(
        &mut self,
        readback: ReadbackImage<'a>,
        tile: &DisplayTile,
    ) -> PipelineResult<RenderTimings<'a>> {
        let (part, counts, source_dims, timings) = readback.into_tile();
        self.counts.add(counts);
        self.source_dims = source_dims;
        let full = self.full;
        let display = self.display.get_or_insert_with(|| part.blank(full));
        let from = tile.core.within(tile.rect);
        match (display, &part) {
            (DisplayBuf::Rgba8(dst), DisplayBuf::Rgba8(src)) => {
                copy_rows(dst, full.0, src, tile.rect.w, from, tile.core, 4);
            }
            (DisplayBuf::Rgb16(dst), DisplayBuf::Rgb16(src)) => {
                copy_rows(dst, full.0, src, tile.rect.w, from, tile.core, 3);
            }
            _ => {
                return Err(PipelineError::Render(
                    "display tiles read back different depths".into(),
                ));
            }
        }
        Ok(timings)
    }

    pub fn finish(self, timings: RenderTimings<'_>) -> PipelineResult<ReadbackImage<'_>> {
        let display = self
            .display
            .ok_or_else(|| PipelineError::Render("display tiles produced no output".into()))?;
        Ok(ReadbackImage::stitched(
            display,
            self.counts,
            self.full,
            self.source_dims,
            timings,
        ))
    }
}

impl DisplayBuf {
    fn blank(&self, (w, h): (u32, u32)) -> Self {
        let texels = w as usize * h as usize;
        match self {
            Self::Rgba8(_) => Self::Rgba8(vec![0; texels * 4]),
            Self::Rgb16(_) => Self::Rgb16(vec![0; texels * 3]),
        }
    }
}

fn copy_rows<T: Copy>(
    dst: &mut [T],
    dst_w: u32,
    src: &[T],
    src_w: u32,
    from: Rect,
    to: Rect,
    channels: usize,
) {
    let row = from.w as usize * channels;
    let start = |x: u32, y: u32, w: u32| (y as usize * w as usize + x as usize) * channels;
    for line in 0..from.h {
        let s = start(from.x, from.y + line, src_w);
        let d = start(to.x, to.y + line, dst_w);
        dst[d..d + row].copy_from_slice(&src[s..s + row]);
    }
}
