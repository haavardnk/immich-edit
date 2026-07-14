import type { EditHistoryEntry } from '$lib/api/edits';
import type { CurveChannel, Edits, GeometryEdits, LensEdits, MaskLayer } from '$lib/types/edits';
import { CURVE_CHANNELS, HSL_BAND_NAMES, isFullCrop, neutralEdits } from '$lib/types/edits';

type NumberFieldDef = {
  kind: 'number';
  section: string;
  label: string;
  get: (edits: Edits) => number;
  precision?: number;
  signed?: boolean;
};

type BooleanFieldDef = {
  kind: 'boolean';
  section: string;
  label: string;
  get: (edits: Edits) => boolean;
};

type FieldDef = NumberFieldDef | BooleanFieldDef;

export type HistoryDetailItem =
  | { kind: 'value'; label: string; before: string; after: string }
  | { kind: 'summary'; text: string };

export type HistoryDetailGroup = {
  key: string;
  label: string;
  items: HistoryDetailItem[];
};

const FIELDS: FieldDef[] = [
  { kind: 'number', section: 'basic', label: 'Exposure', get: (e) => e.basic.exposure_ev, precision: 2, signed: true },
  { kind: 'number', section: 'basic', label: 'Brightness', get: (e) => e.basic.brightness, signed: true },
  { kind: 'number', section: 'basic', label: 'Contrast', get: (e) => e.basic.contrast, signed: true },
  { kind: 'number', section: 'basic', label: 'Vibrance', get: (e) => e.basic.vibrance, signed: true },
  { kind: 'number', section: 'basic', label: 'Saturation', get: (e) => e.basic.saturation, signed: true },
  { kind: 'number', section: 'basic', label: 'Temperature', get: (e) => e.basic.wb_temp, signed: true },
  { kind: 'number', section: 'basic', label: 'Tint', get: (e) => e.basic.wb_tint, signed: true },
  { kind: 'number', section: 'basic', label: 'Texture', get: (e) => e.basic.texture, signed: true },
  { kind: 'number', section: 'basic', label: 'Clarity', get: (e) => e.basic.clarity, signed: true },
  { kind: 'number', section: 'basic', label: 'Dehaze', get: (e) => e.basic.dehaze, signed: true },
  { kind: 'number', section: 'tone', label: 'Highlights', get: (e) => e.tone.highlights, signed: true },
  { kind: 'number', section: 'tone', label: 'Shadows', get: (e) => e.tone.shadows, signed: true },
  { kind: 'number', section: 'tone', label: 'Whites', get: (e) => e.tone.whites, signed: true },
  { kind: 'number', section: 'tone', label: 'Blacks', get: (e) => e.tone.blacks, signed: true },
  { kind: 'number', section: 'detail', label: 'Sharpen Amount', get: (e) => e.detail.sharpen_amount },
  { kind: 'number', section: 'detail', label: 'Sharpen Radius', get: (e) => e.detail.sharpen_radius, precision: 1 },
  { kind: 'number', section: 'detail', label: 'Sharpen Detail', get: (e) => e.detail.sharpen_detail },
  { kind: 'number', section: 'detail', label: 'Sharpen Masking', get: (e) => e.detail.sharpen_masking },
  { kind: 'number', section: 'detail', label: 'Luminance NR', get: (e) => e.detail.luma_nr_amount },
  { kind: 'number', section: 'detail', label: 'Luminance NR Detail', get: (e) => e.detail.luma_nr_detail },
  { kind: 'number', section: 'detail', label: 'Luminance NR Contrast', get: (e) => e.detail.luma_nr_contrast },
  { kind: 'number', section: 'detail', label: 'Color NR', get: (e) => e.detail.color_nr_amount },
  { kind: 'number', section: 'detail', label: 'Color NR Detail', get: (e) => e.detail.color_nr_detail },
  { kind: 'number', section: 'detail', label: 'Color NR Smoothness', get: (e) => e.detail.color_nr_smoothness },
  { kind: 'number', section: 'effects', label: 'Vignette Amount', get: (e) => e.effects.vignette_amount, signed: true },
  { kind: 'number', section: 'effects', label: 'Vignette Midpoint', get: (e) => e.effects.vignette_midpoint },
  { kind: 'number', section: 'effects', label: 'Vignette Feather', get: (e) => e.effects.vignette_feather },
  { kind: 'number', section: 'effects', label: 'Vignette Roundness', get: (e) => e.effects.vignette_roundness, signed: true },
  { kind: 'number', section: 'effects', label: 'Grain Amount', get: (e) => e.effects.grain_amount },
  { kind: 'number', section: 'effects', label: 'Grain Size', get: (e) => e.effects.grain_size },
  { kind: 'number', section: 'effects', label: 'Grain Roughness', get: (e) => e.effects.grain_roughness },
  { kind: 'boolean', section: 'lens', label: 'Profile Corrections', get: (e) => e.lens.profile_enabled },
  { kind: 'boolean', section: 'lens', label: 'Chromatic Aberration', get: (e) => e.lens.ca_enabled },
  { kind: 'boolean', section: 'lens', label: 'Constrain Crop', get: (e) => e.lens.constrain_crop },
  { kind: 'number', section: 'lens', label: 'Lens Distortion', get: (e) => e.lens.distortion_amount },
  { kind: 'number', section: 'lens', label: 'Lens Vignetting', get: (e) => e.lens.vignette_amount },
  { kind: 'number', section: 'color', label: 'Color Balance', get: (e) => e.color.color_grade.balance, signed: true },
  { kind: 'number', section: 'color', label: 'Color Blending', get: (e) => e.color.color_grade.blend }
];

const SECTION_LABELS: Record<string, string> = {
  basic: 'Basic',
  tone: 'Tone',
  color: 'Color',
  detail: 'Detail',
  effects: 'Effects',
  lens: 'Lens'
};

const CURVE_LABELS: Record<CurveChannel, string> = {
  composite: 'RGB',
  r: 'Red',
  g: 'Green',
  b: 'Blue',
  luma: 'Luma'
};

for (const region of ['Shadows', 'Midtones', 'Highlights', 'Global'] as const) {
  const key = region.toLowerCase() as 'shadows' | 'midtones' | 'highlights' | 'global';
  FIELDS.push(
    { kind: 'number', section: 'color', label: `${region} Hue`, get: (e) => e.color.color_grade[key].hue, signed: true },
    { kind: 'number', section: 'color', label: `${region} Saturation`, get: (e) => e.color.color_grade[key].sat, signed: true },
    { kind: 'number', section: 'color', label: `${region} Luminance`, get: (e) => e.color.color_grade[key].lum, signed: true }
  );
}

for (let i = 0; i < HSL_BAND_NAMES.length; i++) {
  const name = HSL_BAND_NAMES[i];
  FIELDS.push(
    { kind: 'number', section: 'color', label: `${name} Hue`, get: (e) => e.color.hsl.bands[i].hue, signed: true },
    { kind: 'number', section: 'color', label: `${name} Saturation`, get: (e) => e.color.hsl.bands[i].sat, signed: true },
    { kind: 'number', section: 'color', label: `${name} Luminance`, get: (e) => e.color.hsl.bands[i].lum, signed: true }
  );
}

export type HistoryLabel = {
  label: string;
  delta?: string;
};

function fmtNumber(value: number, field: NumberFieldDef): string {
  const formatted = value.toFixed(field.precision ?? 0);
  return field.signed && value > 0 ? `+${formatted}` : formatted;
}

function fmtDelta(delta: number, precision: number): string {
  const sign = delta > 0 ? '+' : '';
  return `${sign}${delta.toFixed(precision)}`;
}

function fieldChanged(field: FieldDef, prev: Edits, curr: Edits): boolean {
  if (field.kind === 'boolean') return field.get(prev) !== field.get(curr);
  return Math.abs(field.get(prev) - field.get(curr)) > 1e-4;
}

function snapshots(entry: EditHistoryEntry, previous: EditHistoryEntry | null): [Edits, Edits] {
  const prev = previous && !previous.deleted && previous.edits ? previous.edits : neutralEdits();
  const curr = !entry.deleted && entry.edits ? entry.edits : neutralEdits();
  return [prev, curr];
}

function curvesEqual(a: { x: number; y: number }[], b: { x: number; y: number }[]): boolean {
  return a.length === b.length && a.every((point, i) =>
    Math.abs(point.x - b[i].x) <= 1e-4 && Math.abs(point.y - b[i].y) <= 1e-4
  );
}

function curveItems(prev: Edits, curr: Edits): HistoryDetailItem[] {
  return CURVE_CHANNELS.flatMap((channel) => {
    const before = prev.basic.curves[channel];
    const after = curr.basic.curves[channel];
    if (curvesEqual(before, after)) return [];
    const suffix = before.length === after.length
      ? 'adjusted'
      : `${before.length} → ${after.length} points`;
    return [{ kind: 'summary' as const, text: `${CURVE_LABELS[channel]}: ${suffix}` }];
  });
}

function maskItems(prev: MaskLayer[], curr: MaskLayer[]): HistoryDetailItem[] {
  const prevById = new Map(prev.map((layer) => [layer.id, layer]));
  const currById = new Map(curr.map((layer) => [layer.id, layer]));
  const added = curr.filter((layer) => !prevById.has(layer.id)).length;
  const removed = prev.filter((layer) => !currById.has(layer.id)).length;
  const modified = curr.filter((layer) => {
    const before = prevById.get(layer.id);
    return before !== undefined && JSON.stringify(before) !== JSON.stringify(layer);
  }).length;
  const sameIds = added === 0 && removed === 0;
  const reordered = sameIds && prev.some((layer, i) => layer.id !== curr[i]?.id);
  const changes = [
    added > 0 ? `${added} added` : '',
    removed > 0 ? `${removed} removed` : '',
    modified > 0 ? `${modified} modified` : '',
    reordered ? 'reordered' : ''
  ].filter(Boolean);
  return changes.length > 0 ? [{ kind: 'summary', text: changes.join(', ') }] : [];
}

function aspectLabel(aspect: GeometryEdits['aspect']): string {
  if (aspect.kind === 'ratio') return `${aspect.num}:${aspect.den}`;
  return aspect.kind === 'original' ? 'Original' : 'Free';
}

function geometryItems(prev: GeometryEdits, curr: GeometryEdits): HistoryDetailItem[] {
  const items: HistoryDetailItem[] = [];
  if (prev.rotate !== curr.rotate) items.push({ kind: 'summary', text: `Rotation: ${prev.rotate}° → ${curr.rotate}°` });
  if (Math.abs(prev.rotate_angle - curr.rotate_angle) > 1e-4) {
    items.push({ kind: 'summary', text: `Angle: ${prev.rotate_angle.toFixed(1)}° → ${curr.rotate_angle.toFixed(1)}°` });
  }
  if (prev.flip_h !== curr.flip_h) items.push({ kind: 'summary', text: `Horizontal flip: ${prev.flip_h ? 'On' : 'Off'} → ${curr.flip_h ? 'On' : 'Off'}` });
  if (prev.flip_v !== curr.flip_v) items.push({ kind: 'summary', text: `Vertical flip: ${prev.flip_v ? 'On' : 'Off'} → ${curr.flip_v ? 'On' : 'Off'}` });
  if (JSON.stringify(prev.aspect) !== JSON.stringify(curr.aspect)) {
    items.push({ kind: 'summary', text: `Aspect: ${aspectLabel(prev.aspect)} → ${aspectLabel(curr.aspect)}` });
  }
  if (JSON.stringify(prev.crop) !== JSON.stringify(curr.crop)) {
    const beforeFull = isFullCrop(prev.crop);
    const afterFull = isFullCrop(curr.crop);
    const text = beforeFull ? 'Crop added' : afterFull ? 'Crop removed' : 'Crop adjusted';
    items.push({ kind: 'summary', text });
  }
  return items;
}

const LENS_PROFILE_KEYS = [
  'k1',
  'k2',
  'k3',
  'vk1',
  'vk2',
  'vk3',
  'ca_red_scale_x10000',
  'ca_blue_scale_x10000'
] as const satisfies readonly (keyof LensEdits)[];

function profileChanged(prev: LensEdits, curr: LensEdits): boolean {
  return LENS_PROFILE_KEYS.some((key) => Math.abs(prev[key] - curr[key]) > 1e-4);
}

export function historyDetails(
  entry: EditHistoryEntry,
  previous: EditHistoryEntry | null
): HistoryDetailGroup[] {
  const [prev, curr] = snapshots(entry, previous);
  const groups = new Map<string, HistoryDetailGroup>();
  for (const field of FIELDS) {
    if (!fieldChanged(field, prev, curr)) continue;
    let group = groups.get(field.section);
    if (!group) {
      group = { key: field.section, label: SECTION_LABELS[field.section], items: [] };
      groups.set(field.section, group);
    }
    const before = field.kind === 'boolean'
      ? field.get(prev) ? 'On' : 'Off'
      : fmtNumber(field.get(prev), field);
    const after = field.kind === 'boolean'
      ? field.get(curr) ? 'On' : 'Off'
      : fmtNumber(field.get(curr), field);
    group.items.push({ kind: 'value', label: field.label, before, after });
  }

  const nested: HistoryDetailGroup[] = [
    { key: 'curves', label: 'Curves', items: curveItems(prev, curr) },
    { key: 'masks', label: 'Masks', items: maskItems(prev.masks, curr.masks) },
    { key: 'geometry', label: 'Geometry', items: geometryItems(prev.geometry, curr.geometry) }
  ];
  if (prev.output.tonemap !== curr.output.tonemap) {
    const label = (value: Edits['output']['tonemap']): string => value === 'agx' ? 'AgX' : 'Default';
    nested.push({
      key: 'output',
      label: 'Output',
      items: [{ kind: 'summary', text: `Tonemap: ${label(prev.output.tonemap)} → ${label(curr.output.tonemap)}` }]
    });
  }
  if (profileChanged(prev.lens, curr.lens)) {
    let group = groups.get('lens');
    if (!group) {
      group = { key: 'lens', label: 'Lens', items: [] };
      groups.set('lens', group);
    }
    group.items.push({ kind: 'summary', text: 'Profile data changed' });
  }
  return [...groups.values(), ...nested.filter((group) => group.items.length > 0)];
}

export function historyLabel(
  entry: EditHistoryEntry,
  previous: EditHistoryEntry | null
): HistoryLabel {
  if (entry.deleted) return { label: entry.action ?? 'Reset to original' };
  const curr = entry.edits;
  if (!curr) return { label: entry.action ?? entry.manifest_hash.slice(0, 8) };

  const [prev] = snapshots(entry, previous);
  const scalarDiffs = FIELDS.filter((field) => fieldChanged(field, prev, curr));
  const details = historyDetails(entry, previous);
  const detailCount = details.reduce((count, group) => count + group.items.length, 0);
  if (scalarDiffs.length === 1 && detailCount === 1) {
    const field = scalarDiffs[0];
    if (field.kind === 'number') {
      return {
        label: field.label,
        delta: fmtDelta(field.get(curr) - field.get(prev), field.precision ?? 0)
      };
    }
    return { label: field.label };
  }
  if (entry.action) return { label: entry.action };
  if (detailCount === 0) return { label: entry.manifest_hash.slice(0, 8) };
  return { label: 'Multiple changes' };
}
