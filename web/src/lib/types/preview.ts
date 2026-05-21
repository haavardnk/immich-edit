export interface PreviewMeta {
  asset_id: string;
  width: number;
  height: number;
  renderer: string;
  histogram: Histogram;
}

export interface Histogram {
  r: number[];
  g: number[];
  b: number[];
  l: number[];
}
