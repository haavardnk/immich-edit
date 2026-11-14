import { editsToManifest } from '#lib/edits/manifest.ts';
import { DESKTOP_PANELS } from '#lib/editor/params/catalog.ts';
import type { DesktopPanelId, TouchGroupId } from '#lib/editor/params/types.ts';
import type { EditorTab } from '#lib/stores/ui.svelte.ts';
import { lensBlurActive, type DevelopSection, type Edits } from '#lib/types/edits.ts';

type OpTable<Id extends string> = Readonly<Record<Id, readonly string[]>>;

export const SECTION_OPS: OpTable<DevelopSection> = {
  dcp: ['dcp_hue_sat'],
  white_balance: ['white_balance'],
  tone: ['exposure', 'brightness', 'contrast', 'tone_regions'],
  presence: ['texture', 'clarity', 'dehaze', 'vibrance', 'saturation'],
  curves: ['curves'],
  hsl: ['hsl'],
  color_grading: ['color_grade'],
  bw: ['bw'],
  lut: ['lut_3d'],
  sharpening: ['capture_sharpen', 'sharpen'],
  noise_reduction: ['ai_denoise', 'luma_nr', 'color_nr'],
  lens: ['lens_profile'],
  lens_blur: ['lens_blur'],
  vignette: ['vignette'],
  grain: ['grain']
};

export const TOUCH_VIEW_OPS: Readonly<Record<TouchGroupId, OpTable<string>>> = {
  light: { tone: ['exposure', 'brightness', 'contrast', 'tone_regions'], curve: ['curves'] },
  color: {
    basic: ['white_balance', 'vibrance', 'saturation'],
    mix: ['hsl'],
    grading: ['color_grade'],
    bw: ['bw']
  },
  effects: {
    basic: ['texture', 'clarity', 'dehaze', 'vignette', 'grain'],
    lens_blur: ['lens_blur']
  },
  detail: {
    sharpening: ['capture_sharpen', 'sharpen'],
    noise: ['ai_denoise', 'luma_nr', 'color_nr']
  },
  optics: { corrections: ['lens_profile'] },
  profile: { camera: ['dcp_hue_sat'], lut: ['lut_3d'] }
};

function activeDevelopOps(edits: Edits): Set<string> {
  const active = new Set(Object.keys(editsToManifest(edits).ops));
  if (!lensBlurActive(edits.lens_blur)) active.delete('lens_blur');
  return active;
}

function modifiedIds<Id extends string>(active: ReadonlySet<string>, table: OpTable<Id>): Set<Id> {
  return new Set(
    (Object.keys(table) as Id[]).filter((id) => table[id].some((op) => active.has(op)))
  );
}

export function modifiedDevelopPanels(edits: Edits): Set<DesktopPanelId> {
  const sections = modifiedSections(edits);
  return new Set(
    DESKTOP_PANELS.filter((panel) =>
      panel.sections.some((section) => sections.has(section.id))
    ).map((panel) => panel.id)
  );
}

export function modifiedTouchGroups(edits: Edits): Set<TouchGroupId> {
  const active = activeDevelopOps(edits);
  const groups = Object.keys(TOUCH_VIEW_OPS) as TouchGroupId[];
  return new Set(groups.filter((group) => modifiedIds(active, TOUCH_VIEW_OPS[group]).size > 0));
}

export function modifiedTouchViews(edits: Edits, group: TouchGroupId): Set<string> {
  return modifiedIds(activeDevelopOps(edits), TOUCH_VIEW_OPS[group]);
}

export function modifiedSections(edits: Edits): Set<DevelopSection> {
  return modifiedIds(activeDevelopOps(edits), SECTION_OPS);
}

export function modifiedTabCount(edits: Edits, tab: EditorTab): number {
  if (tab === 'develop') return modifiedDevelopPanels(edits).size;
  if (tab === 'masks') return edits.masks.length;
  if (tab === 'retouch') return edits.retouch.length;
  if (tab === 'geometry') return 'transform' in editsToManifest(edits).ops ? 1 : 0;
  return 0;
}
