import type { Edits } from '$lib/types/edits';
import { neutralEdits, HSL_BAND_NAMES } from '$lib/types/edits';
import type { EditHistoryEntry } from '$lib/api/edits';

type FieldDef = {
  label: string;
  get: (e: Edits) => number;
  precision?: number;
};

const FIELDS: FieldDef[] = [
  { label: 'Exposure', get: (e) => e.basic.exposure_ev, precision: 2 },
  { label: 'Brightness', get: (e) => e.basic.brightness },
  { label: 'Contrast', get: (e) => e.basic.contrast },
  { label: 'Vibrance', get: (e) => e.basic.vibrance },
  { label: 'Saturation', get: (e) => e.basic.saturation },
  { label: 'Temperature', get: (e) => e.basic.wb_temp },
  { label: 'Tint', get: (e) => e.basic.wb_tint },
  { label: 'Texture', get: (e) => e.basic.texture },
  { label: 'Clarity', get: (e) => e.basic.clarity },
  { label: 'Dehaze', get: (e) => e.basic.dehaze },
  { label: 'Highlights', get: (e) => e.tone.highlights },
  { label: 'Shadows', get: (e) => e.tone.shadows },
  { label: 'Whites', get: (e) => e.tone.whites },
  { label: 'Blacks', get: (e) => e.tone.blacks },
  { label: 'Sharpen Amount', get: (e) => e.detail.sharpen_amount },
  { label: 'Sharpen Radius', get: (e) => e.detail.sharpen_radius, precision: 1 },
  { label: 'Sharpen Detail', get: (e) => e.detail.sharpen_detail },
  { label: 'Sharpen Masking', get: (e) => e.detail.sharpen_masking },
  { label: 'Luminance NR', get: (e) => e.detail.luma_nr_amount },
  { label: 'Luminance NR Detail', get: (e) => e.detail.luma_nr_detail },
  { label: 'Luminance NR Contrast', get: (e) => e.detail.luma_nr_contrast },
  { label: 'Color NR', get: (e) => e.detail.color_nr_amount },
  { label: 'Color NR Detail', get: (e) => e.detail.color_nr_detail },
  { label: 'Color NR Smoothness', get: (e) => e.detail.color_nr_smoothness },
  { label: 'Vignette Amount', get: (e) => e.effects.vignette_amount },
  { label: 'Vignette Midpoint', get: (e) => e.effects.vignette_midpoint },
  { label: 'Vignette Feather', get: (e) => e.effects.vignette_feather },
  { label: 'Vignette Roundness', get: (e) => e.effects.vignette_roundness },
  { label: 'Grain Amount', get: (e) => e.effects.grain_amount },
  { label: 'Grain Size', get: (e) => e.effects.grain_size },
  { label: 'Grain Roughness', get: (e) => e.effects.grain_roughness },
  { label: 'Lens Distortion', get: (e) => e.lens.distortion_amount },
  { label: 'Lens Vignetting', get: (e) => e.lens.vignette_amount },
  { label: 'Color Balance', get: (e) => e.color.color_grade.balance },
  { label: 'Color Blending', get: (e) => e.color.color_grade.blend }
];

for (const region of ['Shadows', 'Midtones', 'Highlights', 'Global'] as const) {
  const key = region.toLowerCase() as 'shadows' | 'midtones' | 'highlights' | 'global';
  FIELDS.push(
    { label: `${region} Hue`, get: (e) => e.color.color_grade[key].hue },
    { label: `${region} Saturation`, get: (e) => e.color.color_grade[key].sat },
    { label: `${region} Luminance`, get: (e) => e.color.color_grade[key].lum }
  );
}

for (let i = 0; i < HSL_BAND_NAMES.length; i++) {
  const name = HSL_BAND_NAMES[i];
  FIELDS.push(
    { label: `${name} Hue`, get: (e) => e.color.hsl.bands[i].hue },
    { label: `${name} Saturation`, get: (e) => e.color.hsl.bands[i].sat },
    { label: `${name} Luminance`, get: (e) => e.color.hsl.bands[i].lum }
  );
}

export type HistoryLabel = {
  label: string;
  delta?: string;
};

function fmtDelta(d: number, precision: number): string {
  const sign = d > 0 ? '+' : '';
  return `${sign}${d.toFixed(precision)}`;
}

function diffFields(prev: Edits, curr: Edits): { field: FieldDef; delta: number }[] {
  const out: { field: FieldDef; delta: number }[] = [];
  for (const f of FIELDS) {
    const a = f.get(prev);
    const b = f.get(curr);
    if (Math.abs(a - b) > 1e-4) out.push({ field: f, delta: b - a });
  }
  return out;
}

export function historyLabel(
  entry: EditHistoryEntry,
  previous: EditHistoryEntry | null
): HistoryLabel {
  if (entry.deleted) {
    return { label: entry.action ?? 'Reset to original' };
  }
  const curr = entry.edits;
  if (!curr) return { label: entry.action ?? entry.manifest_hash.slice(0, 8) };

  const prevEdits = previous && !previous.deleted && previous.edits ? previous.edits : neutralEdits();
  const diffs = diffFields(prevEdits, curr);

  if (diffs.length === 1) {
    const { field, delta } = diffs[0];
    return { label: field.label, delta: fmtDelta(delta, field.precision ?? 0) };
  }
  if (entry.action) return { label: entry.action };
  if (diffs.length === 0) return { label: entry.manifest_hash.slice(0, 8) };
  return { label: 'Multiple changes' };
}
