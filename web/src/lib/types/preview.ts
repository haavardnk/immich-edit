export interface PreviewMeta {
  asset_id: string;
  width: number;
  height: number;
  source_w: number;
  source_h: number;
  renderer: string;
  histogram: Histogram;
  linear_histogram?: Histogram;
}

export interface Histogram {
  r: number[];
  g: number[];
  b: number[];
  l: number[];
}
