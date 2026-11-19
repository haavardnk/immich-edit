import { DESKTOP_PANELS } from '#lib/editor/params/catalog.ts';
import type { DevelopSection, Edits } from '#lib/types/edits.ts';
import { sameValue } from '#lib/utils/sameValue.ts';

export type PhotoTool = 'geometry' | 'masks' | 'retouch';

export type CopyTool = DevelopSection | PhotoTool;

export type CopySelection = Record<CopyTool, boolean>;

export interface CopyToolRow {
  key: string;
  label: string;
  tools: readonly CopyTool[];
  children: readonly CopyToolRow[];
}

interface ToolFields {
  copy(to: Edits, from: Edits): void;
  same(a: Edits, b: Edits): boolean;
}

function part<T extends object>(at: (edits: Edits) => T, keys: readonly (keyof T)[]): ToolFields {
  return {
    copy: (to, from) => {
      const target = at(to);
      const source = at(from);
      for (const key of keys) target[key] = source[key];
    },
    same: (a, b) => keys.every((key) => sameValue(at(a)[key], at(b)[key]))
  };
}

function whole<K extends keyof Edits>(key: K): ToolFields {
  return {
    copy: (to, from) => {
      to[key] = from[key];
    },
    same: (a, b) => sameValue(a[key], b[key])
  };
}

function both(...fields: readonly ToolFields[]): ToolFields {
  return {
    copy: (to, from) => fields.forEach((f) => f.copy(to, from)),
    same: (a, b) => fields.every((f) => f.same(a, b))
  };
}

const TOOL_FIELDS: Readonly<Record<CopyTool, ToolFields>> = {
  dcp: part((e) => e.color, ['dcp']),
  white_balance: part((e) => e.basic, ['white_balance']),
  tone: both(
    part((e) => e.basic, ['exposure_ev', 'brightness', 'contrast']),
    whole('tone')
  ),
  presence: part((e) => e.basic, ['texture', 'clarity', 'dehaze', 'vibrance', 'saturation']),
  curves: part((e) => e.basic, ['curves']),
  hsl: part((e) => e.color, ['hsl']),
  color_grading: part((e) => e.color, ['color_grade']),
  bw: part((e) => e.color, ['bw']),
  lut: part((e) => e.color, ['lut_3d']),
  sharpening: part(
    (e) => e.detail,
    [
      'capture_sharpen',
      'capture_corner_boost',
      'sharpen_amount',
      'sharpen_radius',
      'sharpen_detail',
      'sharpen_masking'
    ]
  ),
  noise_reduction: part(
    (e) => e.detail,
    [
      'luma_nr_amount',
      'luma_nr_detail',
      'luma_nr_contrast',
      'color_nr_amount',
      'color_nr_detail',
      'color_nr_smoothness',
      'ai_denoise',
      'ai_denoise_strength',
      'ai_denoise_detail'
    ]
  ),
  lens: whole('lens'),
  lens_blur: part((e) => e.lens_blur, ['amount', 'focus_min', 'focus_max', 'shape', 'boost']),
  vignette: part(
    (e) => e.effects,
    ['vignette_amount', 'vignette_midpoint', 'vignette_feather', 'vignette_roundness']
  ),
  grain: part((e) => e.effects, ['grain_amount', 'grain_size', 'grain_roughness']),
  geometry: whole('geometry'),
  masks: whole('masks'),
  retouch: whole('retouch')
};

function leaf(tool: CopyTool, label: string): CopyToolRow {
  return { key: tool, label, tools: [tool], children: [] };
}

export const DEVELOP_ROWS: readonly CopyToolRow[] = DESKTOP_PANELS.map((panel) => ({
  key: panel.id,
  label: panel.title,
  tools: panel.sections.map((section) => section.id),
  children:
    panel.sections.length > 1
      ? panel.sections.map((section) => leaf(section.id, section.title))
      : []
}));

export const PHOTO_ROWS: readonly CopyToolRow[] = [
  leaf('geometry', 'Geometry & crop'),
  leaf('masks', 'Masks'),
  leaf('retouch', 'Retouch')
];

export const DEVELOP_TOOLS: readonly DevelopSection[] = DESKTOP_PANELS.flatMap((panel) =>
  panel.sections.map((section) => section.id)
);

export const PHOTO_TOOLS: readonly PhotoTool[] = ['geometry', 'masks', 'retouch'];

export const COPY_TOOLS: readonly CopyTool[] = [...DEVELOP_TOOLS, ...PHOTO_TOOLS];

export const TOOL_LABELS: Readonly<Record<CopyTool, string>> = Object.fromEntries(
  [...DEVELOP_ROWS, ...PHOTO_ROWS].flatMap((row) =>
    (row.children.length > 0 ? row.children : [row]).map((leafRow) => [
      leafRow.tools[0],
      leafRow.label
    ])
  )
) as Record<CopyTool, string>;

export function selectionOf(tools: Iterable<CopyTool>): CopySelection {
  const picked = new Set(tools);
  return Object.fromEntries(COPY_TOOLS.map((tool) => [tool, picked.has(tool)])) as CopySelection;
}

export const DEFAULT_COPY_SELECTION: CopySelection = selectionOf(DEVELOP_TOOLS);

export function selectedTools(selection: CopySelection): CopyTool[] {
  return COPY_TOOLS.filter((tool) => selection[tool]);
}

export function withTools(
  selection: CopySelection,
  tools: readonly CopyTool[],
  value: boolean
): CopySelection {
  const next = { ...selection };
  for (const tool of tools) next[tool] = value;
  return next;
}

export function toolsState(
  selection: CopySelection,
  tools: readonly CopyTool[]
): boolean | 'mixed' {
  const on = tools.filter((tool) => selection[tool]).length;
  if (on === 0) return false;
  return on === tools.length ? true : 'mixed';
}

export function toolChanged(tool: CopyTool, before: Edits, after: Edits): boolean {
  return !TOOL_FIELDS[tool].same(before, after);
}

export function applyCopyTools(current: Edits, incoming: Edits, selection: CopySelection): Edits {
  const merged = structuredClone(current);
  const source = structuredClone(incoming);
  for (const tool of selectedTools(selection)) TOOL_FIELDS[tool].copy(merged, source);
  return merged;
}
