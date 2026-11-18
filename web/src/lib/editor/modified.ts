import { editsToManifest } from '#lib/edits/manifest.ts';
import type { DesktopPanelId, TouchGroupId } from '#lib/editor/params/types.ts';
import type { EditorTab } from '#lib/stores/ui.svelte.ts';
import { lensBlurActive, type Edits } from '#lib/types/edits.ts';

type OpTable<Id extends string> = Readonly<Record<Id, readonly string[]>>;

export const DEVELOP_PANEL_OPS: OpTable<DesktopPanelId> = {
  dcp: ['dcp_hue_sat'],
  basic: [
    'white_balance',
    'exposure',
    'brightness',
    'contrast',
    'tone_regions',
    'texture',
    'clarity',
    'dehaze',
    'vibrance',
    'saturation'
  ],
  curves: ['curves'],
  hsl: ['hsl'],
  'color-grading': ['color_grade'],
  bw: ['bw'],
  lut: ['lut_3d'],
  detail: ['capture_sharpen', 'sharpen', 'ai_denoise', 'luma_nr', 'color_nr'],
  lens: ['lens_profile'],
  'lens-blur': ['lens_blur'],
  effects: ['vignette', 'grain']
};

export const TOUCH_TOOL_OPS: OpTable<TouchGroupId> = {
  light: ['exposure', 'brightness', 'contrast', 'tone_regions', 'curves'],
  color: ['white_balance', 'vibrance', 'saturation', 'hsl', 'color_grade', 'bw'],
  effects: ['texture', 'clarity', 'dehaze', 'vignette', 'grain', 'lens_blur'],
  detail: ['capture_sharpen', 'sharpen', 'ai_denoise', 'luma_nr', 'color_nr'],
  optics: ['lens_profile'],
  profile: ['dcp_hue_sat', 'lut_3d']
};

function activeDevelopOps(edits: Edits): Set<string> {
  const active = new Set(Object.keys(editsToManifest(edits).ops));
  if (!lensBlurActive(edits.lens_blur)) active.delete('lens_blur');
  return active;
}

function modifiedIds<Id extends string>(edits: Edits, table: OpTable<Id>): Set<Id> {
  const active = activeDevelopOps(edits);
  return new Set(
    (Object.keys(table) as Id[]).filter((id) => table[id].some((op) => active.has(op)))
  );
}

export function modifiedDevelopPanels(edits: Edits): Set<DesktopPanelId> {
  return modifiedIds(edits, DEVELOP_PANEL_OPS);
}

export function modifiedTouchGroups(edits: Edits): Set<TouchGroupId> {
  return modifiedIds(edits, TOUCH_TOOL_OPS);
}

export function modifiedTabCount(edits: Edits, tab: EditorTab): number {
  if (tab === 'develop') return modifiedDevelopPanels(edits).size;
  if (tab === 'masks') return edits.masks.length;
  if (tab === 'retouch') return edits.retouch.length;
  if (tab === 'geometry') return 'transform' in editsToManifest(edits).ops ? 1 : 0;
  return 0;
}
