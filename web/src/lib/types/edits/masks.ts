export const MASK_COLOR_TOKENS = [
  '#ff3b30',
  '#ff9500',
  '#ffcc00',
  '#34c759',
  '#5ac8fa',
  '#007aff',
  '#af52de',
  '#ff2d55'
] as const;

export const N_MAX_MASK_LAYERS = 8;
export const N_MAX_COMPONENTS_PER_LAYER = 8;
export const N_MAX_TOTAL_COMPONENTS = 32;

export type MaskComponentMode = 'add' | 'subtract' | 'intersect';
export type MaskSource = 'manual' | 'generated';

export interface Vec2f {
  x: number;
  y: number;
}

export type MaskComponentKind =
  | { kind: 'linear'; p0: Vec2f; p1: Vec2f; feather: number }
  | { kind: 'radial'; center: Vec2f; radius_xy: Vec2f; feather: number; angle?: number }
  | { kind: 'brush'; raster_id: string }
  | { kind: 'luma_range'; min: number; max: number; softness: number }
  | {
      kind: 'color_range';
      sample_rgb: [number, number, number];
      tolerance: number;
      softness: number;
    }
  | { kind: 'polygon'; points: Vec2f[]; feather: number };

export interface ClickPointMeta {
  x: number;
  y: number;
  positive: boolean;
}

export interface RangeMeta {
  min: number;
  max: number;
  softness: number;
}

export interface GeneratedMeta {
  model_id: string;
  kind: string;
  prob_raster_id: string;
  class?: string;
  grow: number;
  feather: number;
  painted?: boolean;
  points?: ClickPointMeta[];
  range?: RangeMeta;
}

export interface MaskComponent {
  id: string;
  enabled: boolean;
  mode: MaskComponentMode;
  invert: boolean;
  kind: MaskComponentKind;
  source: MaskSource;
  generated?: GeneratedMeta;
}

export type MaskedEditKey =
  | 'exposure_ev'
  | 'brightness'
  | 'contrast'
  | 'saturation'
  | 'vibrance'
  | 'wb_temp'
  | 'wb_tint'
  | 'highlights'
  | 'shadows'
  | 'whites'
  | 'blacks'
  | 'texture'
  | 'clarity'
  | 'sharpen';

export const MASKED_EDIT_KEYS: readonly MaskedEditKey[] = [
  'exposure_ev',
  'brightness',
  'contrast',
  'saturation',
  'vibrance',
  'wb_temp',
  'wb_tint',
  'highlights',
  'shadows',
  'whites',
  'blacks',
  'texture',
  'clarity',
  'sharpen'
];

export type MaskedEdits = Partial<Record<MaskedEditKey, number>>;

export interface MaskLayer {
  id: string;
  name: string;
  enabled: boolean;
  color: string;
  amount: number;
  invert: boolean;
  components: MaskComponent[];
  edits: MaskedEdits;
}

export function maskedEditsIsZero(m: MaskedEdits): boolean {
  return MASKED_EDIT_KEYS.every((k) => m[k] === undefined || m[k] === 0);
}
