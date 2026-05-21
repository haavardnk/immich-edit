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

export interface Sidecar {
  schema_version: number;
  asset_id: string;
  immich_updated_at: string | null;
  immich_checksum: string | null;
  renderer_version: string;
  edits: Edits;
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
