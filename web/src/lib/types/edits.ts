export interface BasicEdits {
  exposure_ev: number;
  contrast: number;
  saturation: number;
  vibrance: number;
  wb_temp: number;
  wb_tint: number;
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

export const HSL_BAND_COLORS: readonly string[] = [
  '#e53935',
  '#fb8c00',
  '#fdd835',
  '#43a047',
  '#26c6da',
  '#1e88e5',
  '#8e24aa',
  '#d81b60'
];

export interface HslEdits {
  bands: HslBand[];
}

export interface ColorEdits {
  hsl: HslEdits;
}

export interface GeometryEdits {
  rotate: 0 | 90 | 180 | 270;
  flip_h: boolean;
  flip_v: boolean;
}

export interface Edits {
  basic: BasicEdits;
  tone: ToneEdits;
  color: ColorEdits;
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
}

function neutralBands(): HslBand[] {
  return Array.from({ length: HSL_BANDS }, () => ({ hue: 0, sat: 0, lum: 0 }));
}

export function neutralEdits(): Edits {
  return {
    basic: {
      exposure_ev: 0,
      contrast: 0,
      saturation: 0,
      vibrance: 0,
      wb_temp: 0,
      wb_tint: 0
    },
    tone: {
      highlights: 0,
      shadows: 0,
      blacks: 0,
      whites: 0
    },
    color: {
      hsl: { bands: neutralBands() }
    },
    geometry: {
      rotate: 0,
      flip_h: false,
      flip_v: false,
    }
  };
}

export const NEUTRAL_EDITS: Edits = neutralEdits();

function bandsAllZero(bands: HslBand[]): boolean {
  return bands.every((b) => b.hue === 0 && b.sat === 0 && b.lum === 0);
}

export function isIdentity(e: Edits): boolean {
  return (
    e.basic.exposure_ev === 0 &&
    e.basic.contrast === 0 &&
    e.basic.saturation === 0 &&
    e.basic.vibrance === 0 &&
    e.basic.wb_temp === 0 &&
    e.basic.wb_tint === 0 &&
    e.tone.highlights === 0 &&
    e.tone.shadows === 0 &&
    e.tone.blacks === 0 &&
    e.tone.whites === 0 &&
    bandsAllZero(e.color.hsl.bands) &&
    e.geometry.rotate === 0 &&
    !e.geometry.flip_h &&
    !e.geometry.flip_v
  );
}

export function editsToManifest(e: Edits): EditManifest {
  const ops: Record<string, unknown> = {};
  if (e.basic.exposure_ev !== 0) ops.exposure = { ev: e.basic.exposure_ev };
  if (e.basic.contrast !== 0) ops.contrast = { amount: e.basic.contrast };
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
  if (e.basic.wb_temp !== 0 || e.basic.wb_tint !== 0)
    ops.white_balance = { temp: e.basic.wb_temp, tint: e.basic.wb_tint };
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
  return { schema_version: 2, ops };
}

export function manifestToEdits(doc: EditManifest): Edits {
  const edits = neutralEdits();
  const ops = doc.ops ?? {};
  const exposure = ops.exposure as { ev?: number } | undefined;
  if (exposure?.ev !== undefined) edits.basic.exposure_ev = exposure.ev;
  const contrast = ops.contrast as { amount?: number } | undefined;
  if (contrast?.amount !== undefined) edits.basic.contrast = contrast.amount;
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
  const wb = ops.white_balance as { temp?: number; tint?: number } | undefined;
  if (wb?.temp !== undefined) edits.basic.wb_temp = wb.temp;
  if (wb?.tint !== undefined) edits.basic.wb_tint = wb.tint;
  const geom = ops.geometry as
    | { rotate?: number; flip_h?: boolean; flip_v?: boolean }
    | undefined;
  if (geom?.rotate !== undefined)
    edits.geometry.rotate = geom.rotate as GeometryEdits['rotate'];
  if (geom?.flip_h !== undefined) edits.geometry.flip_h = geom.flip_h;
  if (geom?.flip_v !== undefined) edits.geometry.flip_v = geom.flip_v;
  return edits;
}
