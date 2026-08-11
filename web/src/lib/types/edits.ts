import {
  clampPerspective,
  neutralPerspective,
  perspectiveIsIdentity,
  type PerspectiveEdits
} from '$lib/utils/perspective';

export interface CurvePoint {
  x: number;
  y: number;
}

export type CurveChannel = 'composite' | 'r' | 'g' | 'b' | 'luma';

export const CURVE_CHANNELS: readonly CurveChannel[] = ['composite', 'r', 'g', 'b', 'luma'];

export interface CurvesEdits {
  composite: CurvePoint[];
  r: CurvePoint[];
  g: CurvePoint[];
  b: CurvePoint[];
  luma: CurvePoint[];
}

export function identityCurve(): CurvePoint[] {
  return [
    { x: 0, y: 0 },
    { x: 1, y: 1 }
  ];
}

export function neutralCurves(): CurvesEdits {
  return {
    composite: identityCurve(),
    r: identityCurve(),
    g: identityCurve(),
    b: identityCurve(),
    luma: identityCurve()
  };
}

export interface BasicEdits {
  exposure_ev: number;
  brightness: number;
  contrast: number;
  saturation: number;
  vibrance: number;
  wb_temp: number;
  wb_tint: number;
  texture: number;
  clarity: number;
  dehaze: number;
  curves: CurvesEdits;
}

export interface ToneEdits {
  highlights: number;
  shadows: number;
  blacks: number;
  whites: number;
}

export interface HslBand {
  hue: number;
  sat: number;
  lum: number;
}

export const HSL_BANDS = 8;

export const HSL_BAND_NAMES: readonly string[] = [
  'Red',
  'Orange',
  'Yellow',
  'Green',
  'Aqua',
  'Blue',
  'Purple',
  'Magenta'
];

export const HSL_BAND_HUES: readonly number[] = [0, 30, 60, 120, 180, 240, 300, 340];

export const HSL_BAND_COLORS: readonly string[] = HSL_BAND_HUES.map((h) => `hsl(${h}, 70%, 65%)`);

export interface HslEdits {
  bands: HslBand[];
}

export interface ColorGradeRegion {
  hue: number;
  sat: number;
  lum: number;
}

export interface ColorGradeEdits {
  shadows: ColorGradeRegion;
  midtones: ColorGradeRegion;
  highlights: ColorGradeRegion;
  global: ColorGradeRegion;
  balance: number;
  blend: number;
}

export interface Lut3dEdits {
  lut_id: string | null;
  amount: number;
}

export function neutralLut3d(): Lut3dEdits {
  return { lut_id: null, amount: 100 };
}

export function lut3dIsActive(l: Lut3dEdits): boolean {
  return !!l.lut_id && l.amount > 0;
}

export type DcpMode = 'off' | 'auto' | 'profile';
export type DcpIlluminant = 'interpolated' | 'first' | 'second';

export interface DcpEdits {
  mode: DcpMode;
  profile_id: string | null;
  illuminant: DcpIlluminant;
  use_tone_curve: boolean;
  use_base_table: boolean;
  use_look_table: boolean;
  use_baseline_exposure: boolean;
}

export function neutralDcp(): DcpEdits {
  return {
    mode: 'auto',
    profile_id: null,
    illuminant: 'interpolated',
    use_tone_curve: true,
    use_base_table: true,
    use_look_table: true,
    use_baseline_exposure: true
  };
}

export function dcpIsDefault(d: DcpEdits): boolean {
  const neutral = neutralDcp();
  return (
    d.mode === neutral.mode &&
    d.profile_id === neutral.profile_id &&
    d.illuminant === neutral.illuminant &&
    d.use_tone_curve === neutral.use_tone_curve &&
    d.use_base_table === neutral.use_base_table &&
    d.use_look_table === neutral.use_look_table &&
    d.use_baseline_exposure === neutral.use_baseline_exposure
  );
}

export interface ColorEdits {
  hsl: HslEdits;
  color_grade: ColorGradeEdits;
  lut_3d: Lut3dEdits;
  dcp: DcpEdits;
}

export interface DetailEdits {
  capture_sharpen: boolean;
  sharpen_amount: number | null;
  sharpen_radius: number;
  sharpen_detail: number;
  sharpen_masking: number;
  luma_nr_amount: number;
  luma_nr_detail: number;
  luma_nr_contrast: number;
  color_nr_amount: number;
  color_nr_detail: number;
  color_nr_smoothness: number;
}

export const NEUTRAL_DETAIL: DetailEdits = {
  capture_sharpen: true,
  sharpen_amount: null,
  sharpen_radius: 1.0,
  sharpen_detail: 25,
  sharpen_masking: 0,
  luma_nr_amount: 0,
  luma_nr_detail: 50,
  luma_nr_contrast: 0,
  color_nr_amount: 0,
  color_nr_detail: 50,
  color_nr_smoothness: 50
};

export const RAW_SHARPEN_AMOUNT = 40;

export function neutralSharpenAmount(isRaw: boolean): number {
  return isRaw ? RAW_SHARPEN_AMOUNT : 0;
}

export interface EffectsEdits {
  vignette_amount: number;
  vignette_midpoint: number;
  vignette_feather: number;
  vignette_roundness: number;
  grain_amount: number;
  grain_size: number;
  grain_roughness: number;
}

export const NEUTRAL_EFFECTS: EffectsEdits = {
  vignette_amount: 0,
  vignette_midpoint: 50,
  vignette_feather: 50,
  vignette_roundness: 0,
  grain_amount: 0,
  grain_size: 25,
  grain_roughness: 50
};

export interface LensEdits {
  profile_enabled: boolean;
  ca_enabled: boolean;
  constrain_crop: boolean;
  distortion_amount: number;
  vignette_amount: number;
  k1: number;
  k2: number;
  k3: number;
  vk1: number;
  vk2: number;
  vk3: number;
  ca_red_scale_x10000: number;
  ca_blue_scale_x10000: number;
}

export const NEUTRAL_LENS: LensEdits = {
  profile_enabled: false,
  ca_enabled: false,
  constrain_crop: false,
  distortion_amount: 100,
  vignette_amount: 100,
  k1: 0,
  k2: 0,
  k3: 0,
  vk1: 0,
  vk2: 0,
  vk3: 0,
  ca_red_scale_x10000: 0,
  ca_blue_scale_x10000: 0
};

export function lensDistortionActive(l: LensEdits): boolean {
  return l.profile_enabled && l.distortion_amount !== 0 && (l.k1 !== 0 || l.k2 !== 0 || l.k3 !== 0);
}
export function lensVignetteActive(l: LensEdits): boolean {
  return (
    l.profile_enabled && l.vignette_amount !== 0 && (l.vk1 !== 0 || l.vk2 !== 0 || l.vk3 !== 0)
  );
}
export function lensCaActive(l: LensEdits): boolean {
  return l.ca_enabled && (l.ca_red_scale_x10000 !== 0 || l.ca_blue_scale_x10000 !== 0);
}
export function lensIsZero(l: LensEdits): boolean {
  return !lensDistortionActive(l) && !lensVignetteActive(l) && !lensCaActive(l);
}

export interface CropRect {
  x: number;
  y: number;
  w: number;
  h: number;
}

export type AspectLock =
  { kind: 'original' } | { kind: 'free' } | { kind: 'ratio'; num: number; den: number };

export interface GeometryEdits {
  rotate: 0 | 90 | 180 | 270;
  rotate_angle: number;
  flip_h: boolean;
  flip_v: boolean;
  crop: CropRect | null;
  aspect: AspectLock;
  perspective: PerspectiveEdits | null;
}

export const FULL_CROP: CropRect = { x: 0, y: 0, w: 1, h: 1 };

export function isFullCrop(c: CropRect | null): boolean {
  if (!c) return true;
  return (
    Math.abs(c.x) < 1e-4 &&
    Math.abs(c.y) < 1e-4 &&
    Math.abs(c.w - 1) < 1e-4 &&
    Math.abs(c.h - 1) < 1e-4
  );
}

export const N_MAX_MASK_LAYERS = 8;
export const N_MAX_COMPONENTS_PER_LAYER = 8;
export const N_MAX_TOTAL_COMPONENTS = 32;
export const N_MAX_RASTER_SLOTS = 16;

export type MaskComponentMode = 'add' | 'subtract' | 'intersect';
export type MaskSource = 'manual' | 'generated';

export interface Vec2f {
  x: number;
  y: number;
}

export type MaskComponentKind =
  | { kind: 'linear'; p0: Vec2f; p1: Vec2f; feather: number }
  | { kind: 'radial'; center: Vec2f; radius_xy: Vec2f; feather: number }
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

export function maskLayerIsEffective(l: MaskLayer): boolean {
  if (!l.enabled || Math.abs(l.amount) < 1e-6) return false;
  const hasComp = l.components.some((c) => c.enabled);
  return hasComp && !maskedEditsIsZero(l.edits);
}

export type RetouchMode = 'heal' | 'clone';

export const MAX_RETOUCH_STROKES = 64;
export const MAX_RETOUCH_POINTS = 256;

export interface RetouchStroke {
  id: string;
  mode: RetouchMode;
  points: Vec2f[];
  radius: number;
  hardness: number;
  opacity: number;
  source: Vec2f;
  enabled: boolean;
}

export function retouchStrokeIsEffective(s: RetouchStroke): boolean {
  return s.enabled && s.points.length > 0 && s.radius > 0 && s.opacity > 0;
}

export interface Edits {
  basic: BasicEdits;
  tone: ToneEdits;
  color: ColorEdits;
  detail: DetailEdits;
  effects: EffectsEdits;
  lens: LensEdits;
  geometry: GeometryEdits;
  masks: MaskLayer[];
  retouch: RetouchStroke[];
}

export interface EditManifest {
  schema_version: number;
  ops: Record<string, unknown>;
}

export interface EditRecord {
  schema_version: number;
  asset_id: string;
  immich_updated_at: string | null;
  immich_checksum: string | null;
  renderer_version: string;
  manifest: EditManifest;
  updated_at: string;
  hash: string;
}

function neutralBands(): HslBand[] {
  return Array.from({ length: HSL_BANDS }, () => ({ hue: 0, sat: 0, lum: 0 }));
}

function neutralRegion(): ColorGradeRegion {
  return { hue: 0, sat: 0, lum: 0 };
}

function neutralColorGrade(): ColorGradeEdits {
  return {
    shadows: neutralRegion(),
    midtones: neutralRegion(),
    highlights: neutralRegion(),
    global: neutralRegion(),
    balance: 0,
    blend: 0
  };
}

export function neutralEdits(): Edits {
  return {
    basic: {
      exposure_ev: 0,
      brightness: 0,
      contrast: 0,
      saturation: 0,
      vibrance: 0,
      wb_temp: 0,
      wb_tint: 0,
      texture: 0,
      clarity: 0,
      dehaze: 0,
      curves: neutralCurves()
    },
    tone: {
      highlights: 0,
      shadows: 0,
      blacks: 0,
      whites: 0
    },
    color: {
      hsl: { bands: neutralBands() },
      color_grade: neutralColorGrade(),
      lut_3d: neutralLut3d(),
      dcp: neutralDcp()
    },
    detail: { ...NEUTRAL_DETAIL },
    effects: { ...NEUTRAL_EFFECTS },
    lens: { ...NEUTRAL_LENS },
    geometry: {
      rotate: 0,
      rotate_angle: 0,
      flip_h: false,
      flip_v: false,
      crop: null,
      aspect: { kind: 'original' },
      perspective: null
    },
    masks: [],
    retouch: []
  };
}

export function resetDevelopEdits(edits: Edits): Edits {
  const neutral = neutralEdits();
  return {
    ...neutral,
    color: {
      ...neutral.color,
      dcp: { ...edits.color.dcp }
    },
    geometry: edits.geometry
  };
}

export function originalPreviewEdits(edits: Edits): Edits {
  const neutral = neutralEdits();
  return {
    ...edits,
    basic: neutral.basic,
    tone: neutral.tone,
    color: { ...neutral.color, dcp: edits.color.dcp },
    detail: neutral.detail,
    effects: neutral.effects,
    masks: [],
    retouch: []
  };
}

export const NEUTRAL_EDITS: Edits = neutralEdits();

function bandsAllZero(bands: HslBand[]): boolean {
  return bands.every((b) => b.hue === 0 && b.sat === 0 && b.lum === 0);
}

function regionIsZero(r: ColorGradeRegion): boolean {
  return r.sat === 0 && r.lum === 0;
}

function colorGradeIsZero(cg: ColorGradeEdits): boolean {
  return (
    regionIsZero(cg.shadows) &&
    regionIsZero(cg.midtones) &&
    regionIsZero(cg.highlights) &&
    regionIsZero(cg.global)
  );
}

function curvesAreIdentity(pts: CurvePoint[]): boolean {
  return (
    pts.length === 2 &&
    Math.abs(pts[0].x) < 1e-10 &&
    Math.abs(pts[0].y) < 1e-10 &&
    Math.abs(pts[1].x - 1) < 1e-10 &&
    Math.abs(pts[1].y - 1) < 1e-10
  );
}

export function curvesEditsIsIdentity(c: CurvesEdits): boolean {
  return (
    curvesAreIdentity(c.composite) &&
    curvesAreIdentity(c.r) &&
    curvesAreIdentity(c.g) &&
    curvesAreIdentity(c.b) &&
    curvesAreIdentity(c.luma)
  );
}

type NumericKeys<T> = { [K in keyof T]: T[K] extends number ? K : never }[keyof T];

interface NumField {
  key: string;
  get: (e: Edits) => number;
  set: (e: Edits, v: number) => void;
}

interface BoolField {
  key: string;
  get: (e: Edits) => boolean;
  set: (e: Edits, v: boolean) => void;
}

interface NullField {
  key: string;
  get: (e: Edits) => number | null;
  set: (e: Edits, v: number | null) => void;
}

interface FlatOp {
  id: string;
  legacyId?: string;
  nums: NumField[];
  bools?: BoolField[];
  nulls?: NullField[];
  active: (e: Edits) => boolean;
  identity?: (e: Edits) => boolean;
}

function nf(key: string, get: (e: Edits) => number, set: (e: Edits, v: number) => void): NumField {
  return { key, get, set };
}

function bf(
  key: string,
  get: (e: Edits) => boolean,
  set: (e: Edits, v: boolean) => void
): BoolField {
  return { key, get, set };
}

function basicOp(id: string, key: string, field: NumericKeys<BasicEdits>): FlatOp {
  return {
    id,
    nums: [
      nf(
        key,
        (e) => e.basic[field],
        (e, v) => {
          e.basic[field] = v;
        }
      )
    ],
    active: (e) => e.basic[field] !== 0
  };
}

function toneField(field: NumericKeys<ToneEdits>): NumField {
  return nf(
    field,
    (e) => e.tone[field],
    (e, v) => {
      e.tone[field] = v;
    }
  );
}

function detailField(key: string, field: NumericKeys<DetailEdits>): NumField {
  return nf(
    key,
    (e) => e.detail[field],
    (e, v) => {
      e.detail[field] = v;
    }
  );
}

function effectsField(key: string, field: NumericKeys<EffectsEdits>): NumField {
  return nf(
    key,
    (e) => e.effects[field],
    (e, v) => {
      e.effects[field] = v;
    }
  );
}

function lensField(key: string, field: NumericKeys<LensEdits>): NumField {
  return nf(
    key,
    (e) => e.lens[field],
    (e, v) => {
      e.lens[field] = v;
    }
  );
}

const FLAT_OPS: FlatOp[] = [
  basicOp('exposure', 'ev', 'exposure_ev'),
  basicOp('brightness', 'amount', 'brightness'),
  basicOp('contrast', 'amount', 'contrast'),
  basicOp('saturation', 'amount', 'saturation'),
  basicOp('vibrance', 'amount', 'vibrance'),
  basicOp('texture', 'amount', 'texture'),
  basicOp('clarity', 'amount', 'clarity'),
  basicOp('dehaze', 'amount', 'dehaze'),
  {
    id: 'white_balance',
    nums: [
      nf(
        'temp',
        (e) => e.basic.wb_temp,
        (e, v) => {
          e.basic.wb_temp = v;
        }
      ),
      nf(
        'tint',
        (e) => e.basic.wb_tint,
        (e, v) => {
          e.basic.wb_tint = v;
        }
      )
    ],
    active: (e) => e.basic.wb_temp !== 0 || e.basic.wb_tint !== 0
  },
  {
    id: 'tone_regions',
    legacyId: 'highlights_shadows',
    nums: [toneField('highlights'), toneField('shadows'), toneField('blacks'), toneField('whites')],
    active: (e) =>
      e.tone.highlights !== 0 || e.tone.shadows !== 0 || e.tone.blacks !== 0 || e.tone.whites !== 0
  },
  {
    id: 'capture_sharpen',
    nums: [],
    bools: [
      bf(
        'enabled',
        (e) => e.detail.capture_sharpen,
        (e, v) => {
          e.detail.capture_sharpen = v;
        }
      )
    ],
    active: (e) => !e.detail.capture_sharpen
  },
  {
    id: 'sharpen',
    nums: [
      detailField('radius', 'sharpen_radius'),
      detailField('detail', 'sharpen_detail'),
      detailField('masking', 'sharpen_masking')
    ],
    nulls: [
      {
        key: 'amount',
        get: (e) => e.detail.sharpen_amount,
        set: (e, v) => {
          e.detail.sharpen_amount = v;
        }
      }
    ],
    active: (e) =>
      e.detail.sharpen_amount !== null ||
      e.detail.sharpen_radius !== NEUTRAL_DETAIL.sharpen_radius ||
      e.detail.sharpen_detail !== NEUTRAL_DETAIL.sharpen_detail ||
      e.detail.sharpen_masking !== NEUTRAL_DETAIL.sharpen_masking
  },
  {
    id: 'luma_nr',
    nums: [
      detailField('amount', 'luma_nr_amount'),
      detailField('detail', 'luma_nr_detail'),
      detailField('contrast', 'luma_nr_contrast')
    ],
    active: (e) => e.detail.luma_nr_amount !== 0
  },
  {
    id: 'color_nr',
    nums: [
      detailField('amount', 'color_nr_amount'),
      detailField('detail', 'color_nr_detail'),
      detailField('smoothness', 'color_nr_smoothness')
    ],
    active: (e) => e.detail.color_nr_amount !== 0
  },
  {
    id: 'vignette',
    nums: [
      effectsField('amount', 'vignette_amount'),
      effectsField('midpoint', 'vignette_midpoint'),
      effectsField('feather', 'vignette_feather'),
      effectsField('roundness', 'vignette_roundness')
    ],
    active: (e) => e.effects.vignette_amount !== 0
  },
  {
    id: 'grain',
    nums: [
      effectsField('amount', 'grain_amount'),
      effectsField('size', 'grain_size'),
      effectsField('roughness', 'grain_roughness')
    ],
    active: (e) => e.effects.grain_amount !== 0
  },
  {
    id: 'lens_profile',
    nums: [
      lensField('distortion_amount', 'distortion_amount'),
      lensField('vignette_amount', 'vignette_amount'),
      lensField('k1', 'k1'),
      lensField('k2', 'k2'),
      lensField('k3', 'k3'),
      lensField('vk1', 'vk1'),
      lensField('vk2', 'vk2'),
      lensField('vk3', 'vk3'),
      lensField('ca_red', 'ca_red_scale_x10000'),
      lensField('ca_blue', 'ca_blue_scale_x10000')
    ],
    bools: [
      bf(
        'profile_enabled',
        (e) => e.lens.profile_enabled,
        (e, v) => {
          e.lens.profile_enabled = v;
        }
      ),
      bf(
        'ca_enabled',
        (e) => e.lens.ca_enabled,
        (e, v) => {
          e.lens.ca_enabled = v;
        }
      ),
      bf(
        'constrain_crop',
        (e) => e.lens.constrain_crop,
        (e, v) => {
          e.lens.constrain_crop = v;
        }
      )
    ],
    active: (e) =>
      e.lens.profile_enabled ||
      e.lens.ca_enabled ||
      e.lens.constrain_crop ||
      e.lens.k1 !== 0 ||
      e.lens.k2 !== 0 ||
      e.lens.k3 !== 0 ||
      e.lens.vk1 !== 0 ||
      e.lens.vk2 !== 0 ||
      e.lens.vk3 !== 0 ||
      e.lens.ca_red_scale_x10000 !== 0 ||
      e.lens.ca_blue_scale_x10000 !== 0,
    identity: (e) => lensIsZero(e.lens)
  }
];

function flatOpsAreIdentity(e: Edits): boolean {
  return FLAT_OPS.every((op) => (op.identity ? op.identity(e) : !op.active(e)));
}

function encodeFlatOps(e: Edits, ops: Record<string, unknown>): void {
  for (const op of FLAT_OPS) {
    if (!op.active(e)) continue;
    const obj: Record<string, number | boolean | null> = {};
    for (const f of op.nums) obj[f.key] = f.get(e);
    for (const f of op.bools ?? []) obj[f.key] = f.get(e);
    for (const f of op.nulls ?? []) obj[f.key] = f.get(e);
    ops[op.id] = obj;
  }
}

function decodeFlatOps(ops: Record<string, unknown>, e: Edits): void {
  for (const op of FLAT_OPS) {
    const src = ops[op.id] ?? (op.legacyId ? ops[op.legacyId] : undefined);
    if (!src || typeof src !== 'object') continue;
    const raw = src as Record<string, unknown>;
    for (const f of op.nums) {
      const v = raw[f.key];
      if (typeof v === 'number') f.set(e, v);
    }
    for (const f of op.bools ?? []) {
      const v = raw[f.key];
      if (typeof v === 'boolean') f.set(e, v);
    }
    for (const f of op.nulls ?? []) {
      const v = raw[f.key];
      f.set(e, typeof v === 'number' ? v : null);
    }
  }
}

export function isIdentity(e: Edits): boolean {
  return (
    dcpIsDefault(e.color.dcp) &&
    isNonGeometryIdentity(e) &&
    e.geometry.rotate === 0 &&
    Math.abs(e.geometry.rotate_angle) < 1e-4 &&
    !e.geometry.flip_h &&
    !e.geometry.flip_v &&
    isFullCrop(e.geometry.crop) &&
    e.geometry.aspect.kind === 'original' &&
    perspectiveIsIdentity(e.geometry.perspective)
  );
}

export function isNonGeometryIdentity(e: Edits): boolean {
  return (
    flatOpsAreIdentity(e) &&
    curvesEditsIsIdentity(e.basic.curves) &&
    bandsAllZero(e.color.hsl.bands) &&
    colorGradeIsZero(e.color.color_grade) &&
    !lut3dIsActive(e.color.lut_3d) &&
    e.masks.length === 0 &&
    e.retouch.length === 0
  );
}

export function editsToManifest(e: Edits): EditManifest {
  const ops: Record<string, unknown> = {};
  encodeFlatOps(e, ops);
  if (!curvesEditsIsIdentity(e.basic.curves)) {
    const obj: Record<string, [number, number][]> = {};
    const c = e.basic.curves;
    if (!curvesAreIdentity(c.composite)) obj.composite = c.composite.map((p) => [p.x, p.y]);
    if (!curvesAreIdentity(c.r)) obj.r = c.r.map((p) => [p.x, p.y]);
    if (!curvesAreIdentity(c.g)) obj.g = c.g.map((p) => [p.x, p.y]);
    if (!curvesAreIdentity(c.b)) obj.b = c.b.map((p) => [p.x, p.y]);
    if (!curvesAreIdentity(c.luma)) obj.luma = c.luma.map((p) => [p.x, p.y]);
    ops.curves = obj;
  }
  if (!bandsAllZero(e.color.hsl.bands))
    ops.hsl = { bands: e.color.hsl.bands.map((b) => ({ hue: b.hue, sat: b.sat, lum: b.lum })) };
  if (!colorGradeIsZero(e.color.color_grade)) {
    const cg = e.color.color_grade;
    const r = (reg: ColorGradeRegion) => ({ hue: reg.hue, sat: reg.sat, lum: reg.lum });
    ops.color_grade = {
      shadows: r(cg.shadows),
      midtones: r(cg.midtones),
      highlights: r(cg.highlights),
      global: r(cg.global),
      balance: cg.balance,
      blend: cg.blend
    };
  }
  if (lut3dIsActive(e.color.lut_3d))
    ops.lut_3d = { lut_id: e.color.lut_3d.lut_id, amount: e.color.lut_3d.amount };
  if (!dcpIsDefault(e.color.dcp))
    ops.dcp_hue_sat = {
      mode: e.color.dcp.mode,
      profile_id: e.color.dcp.profile_id,
      illuminant: e.color.dcp.illuminant,
      use_tone_curve: e.color.dcp.use_tone_curve,
      use_base_table: e.color.dcp.use_base_table,
      use_look_table: e.color.dcp.use_look_table,
      use_baseline_exposure: e.color.dcp.use_baseline_exposure
    };
  const cropActive = !isFullCrop(e.geometry.crop);
  const angleActive = Math.abs(e.geometry.rotate_angle) > 1e-4;
  const aspectActive = e.geometry.aspect.kind !== 'original';
  const rotateActive = e.geometry.rotate !== 0;
  const flipActive = e.geometry.flip_h || e.geometry.flip_v;
  const perspActive = !perspectiveIsIdentity(e.geometry.perspective);
  if (cropActive || angleActive || aspectActive || rotateActive || flipActive || perspActive) {
    const obj: Record<string, unknown> = {};
    if (rotateActive) obj.rotate = e.geometry.rotate;
    if (e.geometry.flip_h) obj.flip_h = true;
    if (e.geometry.flip_v) obj.flip_v = true;
    if (angleActive) obj.angle = e.geometry.rotate_angle;
    if (e.geometry.crop && cropActive) obj.crop = e.geometry.crop;
    if (perspActive && e.geometry.perspective)
      obj.perspective = clampPerspective(e.geometry.perspective);
    obj.aspect = e.geometry.aspect;
    ops.transform = obj;
  }
  if (e.masks.length > 0) {
    ops.masks = { layers: e.masks };
  }
  if (e.retouch.length > 0) {
    ops.retouch = { strokes: e.retouch };
  }
  return { schema_version: 3, ops };
}

export function manifestToEdits(doc: EditManifest): Edits {
  const edits = neutralEdits();
  const ops = doc.ops ?? {};
  decodeFlatOps(ops, edits);
  const curves = ops.curves as
    | {
        points?: number[][];
        composite?: number[][];
        r?: number[][];
        g?: number[][];
        b?: number[][];
        luma?: number[][];
      }
    | undefined;
  if (curves) {
    const decode = (pts: number[][] | undefined): CurvePoint[] | null => {
      if (!pts || pts.length < 2) return null;
      return pts.map((p) => ({ x: p[0], y: p[1] }));
    };
    if (curves.points) {
      const legacy = decode(curves.points);
      if (legacy) edits.basic.curves.composite = legacy;
    } else {
      const c = decode(curves.composite);
      if (c) edits.basic.curves.composite = c;
      const r = decode(curves.r);
      if (r) edits.basic.curves.r = r;
      const g = decode(curves.g);
      if (g) edits.basic.curves.g = g;
      const b = decode(curves.b);
      if (b) edits.basic.curves.b = b;
      const luma = decode(curves.luma);
      if (luma) edits.basic.curves.luma = luma;
    }
  }
  const hsl = ops.hsl as { bands?: HslBand[] } | undefined;
  if (hsl?.bands) {
    for (let i = 0; i < HSL_BANDS && i < hsl.bands.length; i++) {
      const b = hsl.bands[i];
      if (b.hue !== undefined) edits.color.hsl.bands[i].hue = b.hue;
      if (b.sat !== undefined) edits.color.hsl.bands[i].sat = b.sat;
      if (b.lum !== undefined) edits.color.hsl.bands[i].lum = b.lum;
    }
  }
  const cg = ops.color_grade as
    | {
        shadows?: ColorGradeRegion;
        midtones?: ColorGradeRegion;
        highlights?: ColorGradeRegion;
        global?: ColorGradeRegion;
        balance?: number;
        blend?: number;
      }
    | undefined;
  if (cg) {
    const readRegion = (src: ColorGradeRegion | undefined, dst: ColorGradeRegion) => {
      if (!src) return;
      if (src.hue !== undefined) dst.hue = src.hue;
      if (src.sat !== undefined) dst.sat = src.sat;
      if (src.lum !== undefined) dst.lum = src.lum;
    };
    readRegion(cg.shadows, edits.color.color_grade.shadows);
    readRegion(cg.midtones, edits.color.color_grade.midtones);
    readRegion(cg.highlights, edits.color.color_grade.highlights);
    readRegion(cg.global, edits.color.color_grade.global);
    if (cg.balance !== undefined) edits.color.color_grade.balance = cg.balance;
    if (cg.blend !== undefined) edits.color.color_grade.blend = cg.blend;
  }
  const lut3d = ops.lut_3d as { lut_id?: string; amount?: number } | undefined;
  if (lut3d?.lut_id !== undefined) edits.color.lut_3d.lut_id = lut3d.lut_id;
  if (lut3d?.amount !== undefined) edits.color.lut_3d.amount = lut3d.amount;
  const dcp = ops.dcp_hue_sat as Partial<DcpEdits> | undefined;
  if (dcp) {
    if (dcp.mode !== undefined) edits.color.dcp.mode = dcp.mode;
    if (dcp.profile_id !== undefined) edits.color.dcp.profile_id = dcp.profile_id;
    if (dcp.illuminant !== undefined) edits.color.dcp.illuminant = dcp.illuminant;
    if (dcp.use_tone_curve !== undefined) edits.color.dcp.use_tone_curve = dcp.use_tone_curve;
    if (dcp.use_base_table !== undefined) edits.color.dcp.use_base_table = dcp.use_base_table;
    if (dcp.use_look_table !== undefined) edits.color.dcp.use_look_table = dcp.use_look_table;
    if (dcp.use_baseline_exposure !== undefined)
      edits.color.dcp.use_baseline_exposure = dcp.use_baseline_exposure;
  }
  const transform = ops.transform as
    | {
        rotate?: number;
        flip_h?: boolean;
        flip_v?: boolean;
        angle?: number;
        crop?: CropRect;
        aspect?: AspectLock;
        perspective?: PerspectiveEdits;
      }
    | undefined;
  if (transform?.rotate !== undefined)
    edits.geometry.rotate = transform.rotate as GeometryEdits['rotate'];
  if (transform?.flip_h !== undefined) edits.geometry.flip_h = transform.flip_h;
  if (transform?.flip_v !== undefined) edits.geometry.flip_v = transform.flip_v;
  if (transform?.angle !== undefined) edits.geometry.rotate_angle = transform.angle;
  if (transform?.crop) edits.geometry.crop = transform.crop;
  if (transform?.aspect) edits.geometry.aspect = transform.aspect;
  if (transform?.perspective) {
    const p = clampPerspective({ ...neutralPerspective(), ...transform.perspective });
    edits.geometry.perspective = perspectiveIsIdentity(p) ? null : p;
  }
  const masks = ops.masks as { layers?: unknown[] } | undefined;
  if (masks?.layers) {
    edits.masks = masks.layers
      .map((raw) => parseMaskLayer(raw))
      .filter((l): l is MaskLayer => l !== null);
  }
  const retouch = ops.retouch as { strokes?: unknown[] } | undefined;
  if (retouch?.strokes) {
    edits.retouch = retouch.strokes
      .map((raw) => parseRetouchStroke(raw))
      .filter((s): s is RetouchStroke => s !== null)
      .slice(0, MAX_RETOUCH_STROKES);
  }
  return edits;
}

function parseRetouchStroke(raw: unknown): RetouchStroke | null {
  if (!raw || typeof raw !== 'object') return null;
  const r = raw as Record<string, unknown>;
  if (typeof r.id !== 'string') return null;
  const source = parseVec2f(r.source);
  if (!source) return null;
  const points = (Array.isArray(r.points) ? r.points : [])
    .map((p) => parseVec2f(p))
    .filter((p): p is Vec2f => p !== null)
    .slice(0, MAX_RETOUCH_POINTS);
  if (points.length === 0) return null;
  return {
    id: r.id,
    mode: r.mode === 'clone' ? 'clone' : 'heal',
    points,
    radius: typeof r.radius === 'number' ? r.radius : 0,
    hardness: typeof r.hardness === 'number' ? r.hardness : 0.5,
    opacity: typeof r.opacity === 'number' ? r.opacity : 1,
    source,
    enabled: r.enabled !== false
  };
}

function parseMaskLayer(raw: unknown): MaskLayer | null {
  if (!raw || typeof raw !== 'object') return null;
  const r = raw as Record<string, unknown>;
  if (typeof r.id !== 'string') return null;
  const componentsRaw = Array.isArray(r.components) ? r.components : [];
  const components: MaskComponent[] = [];
  for (const c of componentsRaw) {
    const parsed = parseMaskComponent(c);
    if (parsed) components.push(parsed);
    else return null;
  }
  const edits: MaskedEdits = {};
  const editsRaw = (r.edits ?? {}) as Record<string, unknown>;
  for (const k of MASKED_EDIT_KEYS) {
    const v = editsRaw[k];
    if (typeof v === 'number') edits[k] = v;
  }
  return {
    id: r.id,
    name: typeof r.name === 'string' ? r.name : '',
    enabled: r.enabled !== false,
    color: typeof r.color === 'string' ? r.color : '#ff3b30',
    amount: typeof r.amount === 'number' ? r.amount : 1,
    invert: r.invert === true,
    components,
    edits
  };
}

function parseMaskComponent(raw: unknown): MaskComponent | null {
  if (!raw || typeof raw !== 'object') return null;
  const r = raw as Record<string, unknown>;
  if (typeof r.id !== 'string') return null;
  const kind = parseMaskKind(r.kind);
  if (!kind) return null;
  const mode = r.mode === 'subtract' || r.mode === 'intersect' ? r.mode : 'add';
  const generated = parseGeneratedMeta(r.generated);
  return {
    id: r.id,
    enabled: r.enabled !== false,
    mode,
    invert: r.invert === true,
    kind,
    source: r.source === 'generated' ? 'generated' : 'manual',
    ...(generated ? { generated } : {})
  };
}

function parseGeneratedMeta(raw: unknown): GeneratedMeta | undefined {
  if (!raw || typeof raw !== 'object') return undefined;
  const r = raw as Record<string, unknown>;
  if (typeof r.model_id !== 'string' || typeof r.kind !== 'string') return undefined;
  if (typeof r.prob_raster_id !== 'string') return undefined;
  const points = parseClickPoints(r.points);
  const range = parseRangeMeta(r.range);
  return {
    model_id: r.model_id,
    kind: r.kind,
    prob_raster_id: r.prob_raster_id,
    ...(typeof r.class === 'string' ? { class: r.class } : {}),
    grow: typeof r.grow === 'number' ? r.grow : 0,
    feather: typeof r.feather === 'number' ? r.feather : 0,
    ...(r.painted === true ? { painted: true } : {}),
    ...(points.length > 0 ? { points } : {}),
    ...(range ? { range } : {})
  };
}

function parseRangeMeta(raw: unknown): RangeMeta | undefined {
  if (!raw || typeof raw !== 'object') return undefined;
  const r = raw as Record<string, unknown>;
  if (typeof r.min !== 'number' || typeof r.max !== 'number') return undefined;
  return {
    min: r.min,
    max: r.max,
    softness: typeof r.softness === 'number' ? r.softness : 0
  };
}

function parseClickPoints(raw: unknown): ClickPointMeta[] {
  if (!Array.isArray(raw)) return [];
  const out: ClickPointMeta[] = [];
  for (const item of raw) {
    if (!item || typeof item !== 'object') continue;
    const p = item as Record<string, unknown>;
    if (typeof p.x !== 'number' || typeof p.y !== 'number') continue;
    out.push({ x: p.x, y: p.y, positive: p.positive !== false });
  }
  return out;
}

function parseVec2f(raw: unknown): Vec2f | null {
  if (!raw || typeof raw !== 'object') return null;
  const r = raw as Record<string, unknown>;
  if (typeof r.x !== 'number' || typeof r.y !== 'number') return null;
  return { x: r.x, y: r.y };
}

function parseRgb(raw: unknown): [number, number, number] | null {
  if (!Array.isArray(raw) || raw.length !== 3) return null;
  if (!raw.every((value) => typeof value === 'number' && Number.isFinite(value))) return null;
  return [raw[0], raw[1], raw[2]];
}

function parseMaskKind(raw: unknown): MaskComponentKind | null {
  if (!raw || typeof raw !== 'object') return null;
  const r = raw as Record<string, unknown>;
  if (r.kind === 'linear') {
    const p0 = parseVec2f(r.p0);
    const p1 = parseVec2f(r.p1);
    if (!p0 || !p1) return null;
    return { kind: 'linear', p0, p1, feather: typeof r.feather === 'number' ? r.feather : 0 };
  }
  if (r.kind === 'radial') {
    const center = parseVec2f(r.center);
    const radius_xy = parseVec2f(r.radius_xy);
    if (!center || !radius_xy) return null;
    return {
      kind: 'radial',
      center,
      radius_xy,
      feather: typeof r.feather === 'number' ? r.feather : 0
    };
  }
  if (r.kind === 'brush') {
    if (typeof r.raster_id !== 'string') return null;
    return { kind: 'brush', raster_id: r.raster_id };
  }
  if (r.kind === 'luma_range') {
    if (typeof r.min !== 'number' || typeof r.max !== 'number' || typeof r.softness !== 'number')
      return null;
    return { kind: 'luma_range', min: r.min, max: r.max, softness: r.softness };
  }
  if (r.kind === 'color_range') {
    const sample_rgb = parseRgb(r.sample_rgb);
    if (!sample_rgb || typeof r.tolerance !== 'number' || typeof r.softness !== 'number')
      return null;
    return {
      kind: 'color_range',
      sample_rgb,
      tolerance: r.tolerance,
      softness: r.softness
    };
  }
  if (r.kind === 'polygon') {
    if (!Array.isArray(r.points)) return null;
    const points: Vec2f[] = [];
    for (const raw of r.points) {
      const p = raw as Record<string, unknown>;
      if (typeof p?.x !== 'number' || typeof p?.y !== 'number') return null;
      points.push({ x: p.x, y: p.y });
    }
    return {
      kind: 'polygon',
      points,
      feather: typeof r.feather === 'number' ? r.feather : 0
    };
  }
  return null;
}
