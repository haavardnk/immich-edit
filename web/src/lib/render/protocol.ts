import type { ColorSpaceOpt } from '$lib/api/export';
import type { PreviewMode } from '$lib/api/preview';
import type { Edits } from '$lib/types/edits';
import type { ScopeGrid } from '$lib/types/preview';

export interface RenderView {
  max_edge: number;
  output_color_space?: ColorSpaceOpt;
  preview_mode?: PreviewMode;
  gamut_warn?: boolean;
  clip_warn?: boolean;
  histogram?: boolean;
  scopes?: boolean;
}

export interface SourceInfo {
  width: number;
  height: number;
  frame_width: number;
  frame_height: number;
  is_raw: boolean;
  model: string;
}

export interface RenderInputs {
  sensor_key: string;
  rasters: string[];
  lut: string | null;
}

export interface HistogramBins {
  r: Uint32Array;
  g: Uint32Array;
  b: Uint32Array;
  l: Uint32Array;
}

export interface StageTiming {
  stage: string;
  wall_ms: number;
  gpu_ms?: number;
}

export interface RenderedFrame {
  bitmap: ImageBitmap;
  width: number;
  height: number;
  source_w: number;
  source_h: number;
  histogram?: HistogramBins;
  linear_histogram?: HistogramBins;
  scopes?: { waveform: ScopeGrid; parade: ScopeGrid; vectorscope: ScopeGrid };
  timings: StageTiming[];
}

export type Call =
  | { op: 'init' }
  | { op: 'inputs'; edits: Edits; maxEdge: number }
  | { op: 'setSource'; bytes: ArrayBuffer }
  | { op: 'setRaster'; id: string; width: number; height: number; bytes: ArrayBuffer }
  | { op: 'dropRaster'; id: string }
  | { op: 'setLut'; id: string; bytes: ArrayBuffer }
  | { op: 'dropLut'; id: string }
  | { op: 'setDcp'; bytes: ArrayBuffer | null }
  | { op: 'render'; edits: Edits; view: RenderView };

export interface Request {
  id: number;
  call: Call;
}

export type Reply =
  { id: number; ok: true; value: unknown } | { id: number; ok: false; error: string };
