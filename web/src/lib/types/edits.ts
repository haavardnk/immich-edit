export interface Edits {
  exposure_ev: number;
  contrast: number;
  highlights: number;
  shadows: number;
  saturation: number;
  wb_temp: number;
  wb_tint: number;
  rotate: 0 | 90 | 180 | 270;
  flip_h: boolean;
  flip_v: boolean;
  crop: CropRect | null;
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

export const NEUTRAL_EDITS: Edits = {
  exposure_ev: 0,
  contrast: 0,
  highlights: 0,
  shadows: 0,
  saturation: 0,
  wb_temp: 0,
  wb_tint: 0,
  rotate: 0,
  flip_h: false,
  flip_v: false,
  crop: null
};

export function isIdentity(e: Edits): boolean {
  return (
    e.exposure_ev === 0 &&
    e.contrast === 0 &&
    e.highlights === 0 &&
    e.shadows === 0 &&
    e.saturation === 0 &&
    e.wb_temp === 0 &&
    e.wb_tint === 0 &&
    e.rotate === 0 &&
    !e.flip_h &&
    !e.flip_v &&
    e.crop === null
  );
}

export function editsToManifest(e: Edits): EditManifest {
  const ops: Record<string, unknown> = {};
  if (e.exposure_ev !== 0) ops.exposure = { ev: e.exposure_ev };
  if (e.contrast !== 0) ops.contrast = { amount: e.contrast };
  if (e.highlights !== 0 || e.shadows !== 0)
    ops.highlights_shadows = { highlights: e.highlights, shadows: e.shadows };
  if (e.saturation !== 0) ops.saturation = { amount: e.saturation };
  if (e.wb_temp !== 0 || e.wb_tint !== 0)
    ops.white_balance = { temp: e.wb_temp, tint: e.wb_tint };
  if (e.rotate !== 0 || e.flip_h || e.flip_v || e.crop !== null)
    ops.geometry = {
      rotate: e.rotate,
      flip_h: e.flip_h,
      flip_v: e.flip_v,
      crop: e.crop
    };
  return { schema_version: 2, ops };
}

export function manifestToEdits(doc: EditManifest): Edits {
  const edits: Edits = { ...NEUTRAL_EDITS };
  const ops = doc.ops ?? {};
  const exposure = ops.exposure as { ev?: number } | undefined;
  if (exposure?.ev !== undefined) edits.exposure_ev = exposure.ev;
  const contrast = ops.contrast as { amount?: number } | undefined;
  if (contrast?.amount !== undefined) edits.contrast = contrast.amount;
  const hs = ops.highlights_shadows as
    | { highlights?: number; shadows?: number }
    | undefined;
  if (hs?.highlights !== undefined) edits.highlights = hs.highlights;
  if (hs?.shadows !== undefined) edits.shadows = hs.shadows;
  const sat = ops.saturation as { amount?: number } | undefined;
  if (sat?.amount !== undefined) edits.saturation = sat.amount;
  const wb = ops.white_balance as { temp?: number; tint?: number } | undefined;
  if (wb?.temp !== undefined) edits.wb_temp = wb.temp;
  if (wb?.tint !== undefined) edits.wb_tint = wb.tint;
  const geom = ops.geometry as
    | { rotate?: number; flip_h?: boolean; flip_v?: boolean; crop?: CropRect | null }
    | undefined;
  if (geom?.rotate !== undefined) edits.rotate = geom.rotate as Edits['rotate'];
  if (geom?.flip_h !== undefined) edits.flip_h = geom.flip_h;
  if (geom?.flip_v !== undefined) edits.flip_v = geom.flip_v;
  if (geom?.crop !== undefined) edits.crop = geom.crop;
  return edits;
}
