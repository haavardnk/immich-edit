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

export const HSL_BAND_COLORS: readonly string[] = HSL_BAND_HUES.map(
  (h) => `hsl(${h}, 70%, 65%)`
);

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

export interface ColorEdits {
  hsl: HslEdits;
  color_grade: ColorGradeEdits;
}

export interface DetailEdits {
  sharpen_amount: number;
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
  sharpen_amount: 0,
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

export interface CropRect {
  x: number;
  y: number;
  w: number;
  h: number;
}

export type AspectLock =
  | { kind: 'original' }
  | { kind: 'free' }
  | { kind: 'ratio'; num: number; den: number };

export interface GeometryEdits {
  rotate: 0 | 90 | 180 | 270;
  rotate_angle: number;
  flip_h: boolean;
  flip_v: boolean;
  crop: CropRect | null;
  aspect: AspectLock;
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

export interface Edits {
  basic: BasicEdits;
  tone: ToneEdits;
  color: ColorEdits;
  detail: DetailEdits;
  effects: EffectsEdits;
  geometry: GeometryEdits;
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
      color_grade: neutralColorGrade()
    },
    detail: { ...NEUTRAL_DETAIL },
    effects: { ...NEUTRAL_EFFECTS },
    geometry: {
      rotate: 0,
      rotate_angle: 0,
      flip_h: false,
      flip_v: false,
      crop: null,
      aspect: { kind: 'original' }
    }
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

export function isIdentity(e: Edits): boolean {
  return isNonGeometryIdentity(e) &&
    e.geometry.rotate === 0 &&
    Math.abs(e.geometry.rotate_angle) < 1e-4 &&
    !e.geometry.flip_h &&
    !e.geometry.flip_v &&
    isFullCrop(e.geometry.crop) &&
    e.geometry.aspect.kind === 'original';
}

export function isNonGeometryIdentity(e: Edits): boolean {
  return (
    e.basic.exposure_ev === 0 &&
    e.basic.contrast === 0 &&
    e.basic.saturation === 0 &&
    e.basic.vibrance === 0 &&
    e.basic.wb_temp === 0 &&
    e.basic.wb_tint === 0 &&
    e.basic.texture === 0 &&
    e.basic.clarity === 0 &&
    e.basic.dehaze === 0 &&
    curvesEditsIsIdentity(e.basic.curves) &&
    e.tone.highlights === 0 &&
    e.tone.shadows === 0 &&
    e.tone.blacks === 0 &&
    e.tone.whites === 0 &&
    bandsAllZero(e.color.hsl.bands) &&
    colorGradeIsZero(e.color.color_grade) &&
    e.detail.sharpen_amount === 0 &&
    e.detail.luma_nr_amount === 0 &&
    e.detail.color_nr_amount === 0 &&
    e.effects.vignette_amount === 0 &&
    e.effects.grain_amount === 0
  );
}

export function editsToManifest(e: Edits): EditManifest {
  const ops: Record<string, unknown> = {};
  if (e.basic.exposure_ev !== 0) ops.exposure = { ev: e.basic.exposure_ev };
  if (e.basic.contrast !== 0) ops.contrast = { amount: e.basic.contrast };
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
  if (
    e.tone.highlights !== 0 ||
    e.tone.shadows !== 0 ||
    e.tone.blacks !== 0 ||
    e.tone.whites !== 0
  )
    ops.tone_regions = {
      highlights: e.tone.highlights,
      shadows: e.tone.shadows,
      blacks: e.tone.blacks,
      whites: e.tone.whites
    };
  if (e.basic.saturation !== 0) ops.saturation = { amount: e.basic.saturation };
  if (e.basic.vibrance !== 0) ops.vibrance = { amount: e.basic.vibrance };
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
  if (e.basic.wb_temp !== 0 || e.basic.wb_tint !== 0)
    ops.white_balance = { temp: e.basic.wb_temp, tint: e.basic.wb_tint };
  if (e.basic.texture !== 0) ops.texture = { amount: e.basic.texture };
  if (e.basic.clarity !== 0) ops.clarity = { amount: e.basic.clarity };
  if (e.basic.dehaze !== 0) ops.dehaze = { amount: e.basic.dehaze };
  if (e.detail.sharpen_amount !== 0)
    ops.sharpen = {
      amount: e.detail.sharpen_amount,
      radius: e.detail.sharpen_radius,
      detail: e.detail.sharpen_detail,
      masking: e.detail.sharpen_masking
    };
  if (e.detail.luma_nr_amount !== 0)
    ops.luma_nr = {
      amount: e.detail.luma_nr_amount,
      detail: e.detail.luma_nr_detail,
      contrast: e.detail.luma_nr_contrast
    };
  if (e.detail.color_nr_amount !== 0)
    ops.color_nr = {
      amount: e.detail.color_nr_amount,
      detail: e.detail.color_nr_detail,
      smoothness: e.detail.color_nr_smoothness
    };
  if (e.effects.vignette_amount !== 0)
    ops.vignette = {
      amount: e.effects.vignette_amount,
      midpoint: e.effects.vignette_midpoint,
      feather: e.effects.vignette_feather,
      roundness: e.effects.vignette_roundness
    };
  if (e.effects.grain_amount !== 0)
    ops.grain = {
      amount: e.effects.grain_amount,
      size: e.effects.grain_size,
      roughness: e.effects.grain_roughness
    };
  if (
    e.geometry.rotate !== 0 ||
    e.geometry.flip_h ||
    e.geometry.flip_v
  )
    ops.geometry = {
      rotate: e.geometry.rotate,
      flip_h: e.geometry.flip_h,
      flip_v: e.geometry.flip_v,
    };
  const cropActive = !isFullCrop(e.geometry.crop);
  const angleActive = Math.abs(e.geometry.rotate_angle) > 1e-4;
  const aspectActive = e.geometry.aspect.kind !== 'original';
  if (cropActive || angleActive || aspectActive) {
    const obj: Record<string, unknown> = { aspect: e.geometry.aspect };
    if (angleActive) obj.angle = e.geometry.rotate_angle;
    if (e.geometry.crop && cropActive) obj.crop = e.geometry.crop;
    ops.crop_rotate = obj;
  }
  return { schema_version: 3, ops };
}

export function manifestToEdits(doc: EditManifest): Edits {
  const edits = neutralEdits();
  const ops = doc.ops ?? {};
  const exposure = ops.exposure as { ev?: number } | undefined;
  if (exposure?.ev !== undefined) edits.basic.exposure_ev = exposure.ev;
  const contrast = ops.contrast as { amount?: number } | undefined;
  if (contrast?.amount !== undefined) edits.basic.contrast = contrast.amount;
  const curves = ops.curves as
    | { points?: number[][]; composite?: number[][]; r?: number[][]; g?: number[][]; b?: number[][]; luma?: number[][] }
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
  const tr = (ops.tone_regions ?? ops.highlights_shadows) as
    | { highlights?: number; shadows?: number; blacks?: number; whites?: number }
    | undefined;
  if (tr?.highlights !== undefined) edits.tone.highlights = tr.highlights;
  if (tr?.shadows !== undefined) edits.tone.shadows = tr.shadows;
  if (tr?.blacks !== undefined) edits.tone.blacks = tr.blacks;
  if (tr?.whites !== undefined) edits.tone.whites = tr.whites;
  const sat = ops.saturation as { amount?: number } | undefined;
  if (sat?.amount !== undefined) edits.basic.saturation = sat.amount;
  const vib = ops.vibrance as { amount?: number } | undefined;
  if (vib?.amount !== undefined) edits.basic.vibrance = vib.amount;
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
  const wb = ops.white_balance as { temp?: number; tint?: number } | undefined;
  if (wb?.temp !== undefined) edits.basic.wb_temp = wb.temp;
  if (wb?.tint !== undefined) edits.basic.wb_tint = wb.tint;
  const tex = ops.texture as { amount?: number } | undefined;
  if (tex?.amount !== undefined) edits.basic.texture = tex.amount;
  const cla = ops.clarity as { amount?: number } | undefined;
  if (cla?.amount !== undefined) edits.basic.clarity = cla.amount;
  const dhz = ops.dehaze as { amount?: number } | undefined;
  if (dhz?.amount !== undefined) edits.basic.dehaze = dhz.amount;
  const sh = ops.sharpen as
    | { amount?: number; radius?: number; detail?: number; masking?: number }
    | undefined;
  if (sh?.amount !== undefined) edits.detail.sharpen_amount = sh.amount;
  if (sh?.radius !== undefined) edits.detail.sharpen_radius = sh.radius;
  if (sh?.detail !== undefined) edits.detail.sharpen_detail = sh.detail;
  if (sh?.masking !== undefined) edits.detail.sharpen_masking = sh.masking;
  const lnr = ops.luma_nr as
    | { amount?: number; detail?: number; contrast?: number }
    | undefined;
  if (lnr?.amount !== undefined) edits.detail.luma_nr_amount = lnr.amount;
  if (lnr?.detail !== undefined) edits.detail.luma_nr_detail = lnr.detail;
  if (lnr?.contrast !== undefined) edits.detail.luma_nr_contrast = lnr.contrast;
  const cnr = ops.color_nr as
    | { amount?: number; detail?: number; smoothness?: number }
    | undefined;
  if (cnr?.amount !== undefined) edits.detail.color_nr_amount = cnr.amount;
  if (cnr?.detail !== undefined) edits.detail.color_nr_detail = cnr.detail;
  if (cnr?.smoothness !== undefined) edits.detail.color_nr_smoothness = cnr.smoothness;
  const vig = ops.vignette as
    | { amount?: number; midpoint?: number; feather?: number; roundness?: number }
    | undefined;
  if (vig?.amount !== undefined) edits.effects.vignette_amount = vig.amount;
  if (vig?.midpoint !== undefined) edits.effects.vignette_midpoint = vig.midpoint;
  if (vig?.feather !== undefined) edits.effects.vignette_feather = vig.feather;
  if (vig?.roundness !== undefined) edits.effects.vignette_roundness = vig.roundness;
  const gr = ops.grain as
    | { amount?: number; size?: number; roughness?: number }
    | undefined;
  if (gr?.amount !== undefined) edits.effects.grain_amount = gr.amount;
  if (gr?.size !== undefined) edits.effects.grain_size = gr.size;
  if (gr?.roughness !== undefined) edits.effects.grain_roughness = gr.roughness;
  const geom = ops.geometry as
    | { rotate?: number; flip_h?: boolean; flip_v?: boolean }
    | undefined;
  if (geom?.rotate !== undefined)
    edits.geometry.rotate = geom.rotate as GeometryEdits['rotate'];
  if (geom?.flip_h !== undefined) edits.geometry.flip_h = geom.flip_h;
  if (geom?.flip_v !== undefined) edits.geometry.flip_v = geom.flip_v;
  const cr = ops.crop_rotate as
    | { angle?: number; crop?: CropRect; aspect?: AspectLock }
    | undefined;
  if (cr?.angle !== undefined) edits.geometry.rotate_angle = cr.angle;
  if (cr?.crop) edits.geometry.crop = cr.crop;
  if (cr?.aspect) edits.geometry.aspect = cr.aspect;
  return edits;
}
