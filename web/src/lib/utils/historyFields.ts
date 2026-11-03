import { sectionLabel } from '#lib/editor/params/define.ts';
import { DESKTOP_SECTIONS, SECTIONS } from '#lib/editor/params/catalog.ts';
import type { ParamDef } from '#lib/editor/params/types.ts';
import type { DevelopSection, Edits } from '#lib/types/edits.ts';

type ToggleField = {
  kind: 'boolean' | 'tri';
  section: DevelopSection;
  label: string;
  get: (edits: Edits) => boolean | null;
};

type NumberField = { kind: 'number'; section: DevelopSection; label: string; param: ParamDef };

export type HistoryField = NumberField | ToggleField;

const TOGGLES: readonly ToggleField[] = [
  {
    kind: 'boolean',
    section: 'dcp',
    label: 'DCP Base Table',
    get: (e) => e.color.dcp.use_base_table
  },
  {
    kind: 'boolean',
    section: 'dcp',
    label: 'DCP Tone Curve',
    get: (e) => e.color.dcp.use_tone_curve
  },
  {
    kind: 'boolean',
    section: 'dcp',
    label: 'DCP Look Table',
    get: (e) => e.color.dcp.use_look_table
  },
  {
    kind: 'boolean',
    section: 'dcp',
    label: 'DCP Baseline Exposure',
    get: (e) => e.color.dcp.use_baseline_exposure
  },
  { kind: 'boolean', section: 'bw', label: 'Black & White', get: (e) => e.color.bw.enabled },
  {
    kind: 'boolean',
    section: 'sharpening',
    label: 'Capture Sharpening',
    get: (e) => e.detail.capture_sharpen
  },
  {
    kind: 'boolean',
    section: 'noise_reduction',
    label: 'AI Denoise',
    get: (e) => e.detail.ai_denoise
  },
  {
    kind: 'tri',
    section: 'lens',
    label: 'Profile Corrections',
    get: (e) => e.lens.profile_enabled
  },
  {
    kind: 'boolean',
    section: 'lens',
    label: 'Chromatic Aberration',
    get: (e) => e.lens.ca_enabled
  },
  { kind: 'boolean', section: 'lens', label: 'Constrain Crop', get: (e) => e.lens.constrain_crop }
];

export const HISTORY_FIELDS: readonly HistoryField[] = DESKTOP_SECTIONS.flatMap((section) => [
  ...TOGGLES.filter((toggle) => toggle.section === section.id),
  ...section.rows.flat().map((param): NumberField => ({
    kind: 'number',
    section: section.id,
    label: param.history,
    param
  }))
]);

export function historyGroupLabel(key: DevelopSection): string {
  return sectionLabel(SECTIONS[key]);
}

function signed(param: ParamDef, value: number): string {
  const text = param.format(value);
  return value > 0 ? `+${text}` : text;
}

export function fieldChanged(
  field: HistoryField,
  prev: Edits,
  curr: Edits,
  isRaw: boolean
): boolean {
  if (field.kind !== 'number') return field.get(prev) !== field.get(curr);
  return Math.abs(field.param.get(prev, isRaw) - field.param.get(curr, isRaw)) > 1e-4;
}

export function formatField(field: HistoryField, edits: Edits, isRaw: boolean): string {
  if (field.kind === 'number') {
    const value = field.param.get(edits, isRaw);
    return field.param.min < 0 ? signed(field.param, value) : field.param.format(value);
  }
  const value = field.get(edits);
  if (value === null) return 'Auto';
  return value ? 'On' : 'Off';
}

export function fieldDelta(
  field: HistoryField,
  prev: Edits,
  curr: Edits,
  isRaw: boolean
): string | undefined {
  if (field.kind !== 'number') return undefined;
  return signed(field.param, field.param.get(curr, isRaw) - field.param.get(prev, isRaw));
}
