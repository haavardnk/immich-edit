export interface BasicEdits {
  exposure_ev: number;
  contrast: number;
  saturation: number;
  wb_temp: number;
  wb_tint: number;
}

export interface ToneEdits {
  highlights: number;
  shadows: number;
}

export interface GeometryEdits {
  rotate: 0 | 90 | 180 | 270;
  flip_h: boolean;
  flip_v: boolean;
  crop: CropRect | null;
}

export interface Edits {
  basic: BasicEdits;
  tone: ToneEdits;
  geometry: GeometryEdits;
}

export interface CropRect {
  x: number;
  y: number;
  width: number;
  height: number;
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

export function neutralEdits(): Edits {
  return {
    basic: {
      exposure_ev: 0,
      contrast: 0,
      saturation: 0,
      wb_temp: 0,
      wb_tint: 0
    },
    tone: {
      highlights: 0,
      shadows: 0
    },
    geometry: {
      rotate: 0,
      flip_h: false,
      flip_v: false,
      crop: null
    }
  };
}

export const NEUTRAL_EDITS: Edits = neutralEdits();

export function isIdentity(e: Edits): boolean {
  return (
    e.basic.exposure_ev === 0 &&
    e.basic.contrast === 0 &&
    e.basic.saturation === 0 &&
    e.basic.wb_temp === 0 &&
    e.basic.wb_tint === 0 &&
    e.tone.highlights === 0 &&
    e.tone.shadows === 0 &&
    e.geometry.rotate === 0 &&
    !e.geometry.flip_h &&
    !e.geometry.flip_v &&
    e.geometry.crop === null
  );
}

export function editsToManifest(e: Edits): EditManifest {
  const ops: Record<string, unknown> = {};
  if (e.basic.exposure_ev !== 0) ops.exposure = { ev: e.basic.exposure_ev };
  if (e.basic.contrast !== 0) ops.contrast = { amount: e.basic.contrast };
  if (e.tone.highlights !== 0 || e.tone.shadows !== 0)
    ops.highlights_shadows = { highlights: e.tone.highlights, shadows: e.tone.shadows };
  if (e.basic.saturation !== 0) ops.saturation = { amount: e.basic.saturation };
  if (e.basic.wb_temp !== 0 || e.basic.wb_tint !== 0)
    ops.white_balance = { temp: e.basic.wb_temp, tint: e.basic.wb_tint };
  if (
    e.geometry.rotate !== 0 ||
    e.geometry.flip_h ||
    e.geometry.flip_v ||
    e.geometry.crop !== null
  )
    ops.geometry = {
      rotate: e.geometry.rotate,
      flip_h: e.geometry.flip_h,
      flip_v: e.geometry.flip_v,
      crop: e.geometry.crop
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
  const hs = ops.highlights_shadows as
    | { highlights?: number; shadows?: number }
    | undefined;
  if (hs?.highlights !== undefined) edits.tone.highlights = hs.highlights;
  if (hs?.shadows !== undefined) edits.tone.shadows = hs.shadows;
  const sat = ops.saturation as { amount?: number } | undefined;
  if (sat?.amount !== undefined) edits.basic.saturation = sat.amount;
  const wb = ops.white_balance as { temp?: number; tint?: number } | undefined;
  if (wb?.temp !== undefined) edits.basic.wb_temp = wb.temp;
  if (wb?.tint !== undefined) edits.basic.wb_tint = wb.tint;
  const geom = ops.geometry as
    | { rotate?: number; flip_h?: boolean; flip_v?: boolean; crop?: CropRect | null }
    | undefined;
  if (geom?.rotate !== undefined)
    edits.geometry.rotate = geom.rotate as GeometryEdits['rotate'];
  if (geom?.flip_h !== undefined) edits.geometry.flip_h = geom.flip_h;
  if (geom?.flip_v !== undefined) edits.geometry.flip_v = geom.flip_v;
  if (geom?.crop !== undefined) edits.geometry.crop = geom.crop;
  return edits;
}
