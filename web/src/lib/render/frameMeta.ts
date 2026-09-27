import type { HistogramBins, RenderedFrame } from './protocol';
import type { Histogram, PreviewMeta } from '$lib/types/preview';

const EMPTY_BINS: Histogram = { r: [], g: [], b: [], l: [] };

function histogram(bins: HistogramBins): Histogram {
  return {
    r: Array.from(bins.r),
    g: Array.from(bins.g),
    b: Array.from(bins.b),
    l: Array.from(bins.l)
  };
}

export function frameMeta(
  assetId: string,
  frame: RenderedFrame,
  isRaw: boolean,
  renderer: string
): PreviewMeta {
  return {
    asset_id: assetId,
    width: frame.width,
    height: frame.height,
    source_w: frame.source_w,
    source_h: frame.source_h,
    renderer,
    is_raw: isRaw,
    histogram: frame.histogram ? histogram(frame.histogram) : EMPTY_BINS,
    linear_histogram: frame.linear_histogram ? histogram(frame.linear_histogram) : undefined,
    has_scopes: !!frame.scopes
  };
}
