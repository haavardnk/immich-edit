use super::GpuRenderer;
use super::display::DisplayFrame;
use crate::PipelineResult;
use crate::histogram::Histogram;
use crate::scopes::ScopeGrids;
use crate::timing::StageTiming;

pub struct DisplayMeta {
    pub dims: (u32, u32),
    pub source_dims: (u32, u32),
    pub histogram: Option<Histogram>,
    pub linear_histogram: Option<Histogram>,
    pub scopes: Option<ScopeGrids>,
    pub timings: Vec<StageTiming>,
}

impl GpuRenderer {
    #[expect(
        clippy::await_holding_lock,
        reason = "the frame owns its pooled targets until the bins are read, and the web pool refuses a second frame"
    )]
    pub async fn finish_display(&self, frame: DisplayFrame<'_>) -> PipelineResult<DisplayMeta> {
        let DisplayFrame {
            encoder,
            targets,
            slot,
            bins,
            dims,
            source_dims,
            atlases,
            timings,
            sharpen,
            scratch,
            retained,
        } = frame;
        self.ctx.queue.submit(Some(encoder.finish()));
        drop((slot, sharpen, scratch, retained));
        let counts = self.read_meta_counts_async(&targets[0], bins).await?;
        drop(targets);
        let [preview_atlas, layer_atlas] = atlases;
        self.release_mask_atlas(preview_atlas);
        self.release_mask_atlas(layer_atlas);
        let (histogram, linear_histogram) = counts.histograms();
        let scopes = counts.scopes(timings.clock());
        Ok(DisplayMeta {
            dims,
            source_dims,
            histogram,
            linear_histogram,
            scopes,
            timings: timings.finish_async().await,
        })
    }
}
