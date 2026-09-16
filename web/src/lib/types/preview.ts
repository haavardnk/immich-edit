export interface PreviewMeta {
  asset_id: string;
  width: number;
  height: number;
  source_w: number;
  source_h: number;
  renderer: string;
  is_raw: boolean;
  histogram: Histogram;
  linear_histogram?: Histogram;
  has_scopes: boolean;
}

export type ScopeKind = 'waveform' | 'parade' | 'vectorscope';

export interface ScopeGrid {
  kind: ScopeKind;
  width: number;
  height: number;
  channels: number;
  maxCount: number;
  data: Uint8Array;
}

export interface Histogram {
  r: number[];
  g: number[];
  b: number[];
  l: number[];
}
