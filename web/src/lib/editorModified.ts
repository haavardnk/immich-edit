import { editsToManifest } from '$lib/edits/manifest';
import type { EditorTab } from '$lib/stores/ui.svelte';
import type { Edits } from '$lib/types/edits';

const DEVELOP_PANEL_OPS: Record<string, readonly string[]> = {
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
  lut: ['lut_3d'],
  detail: ['capture_sharpen', 'sharpen', 'luma_nr', 'color_nr'],
  lens: ['lens_profile'],
  effects: ['vignette', 'grain']
};

export function modifiedDevelopPanels(edits: Edits): Set<string> {
  const active = new Set(Object.keys(editsToManifest(edits).ops));
  return new Set(
    Object.entries(DEVELOP_PANEL_OPS)
      .filter(([, opIds]) => opIds.some((id) => active.has(id)))
      .map(([panelId]) => panelId)
  );
}

export function modifiedTabCount(edits: Edits, tab: EditorTab): number {
  if (tab === 'develop') return modifiedDevelopPanels(edits).size;
  if (tab === 'masks') return edits.masks.length;
  if (tab === 'retouch') return edits.retouch.length;
  if (tab === 'geometry') return 'transform' in editsToManifest(edits).ops ? 1 : 0;
  return 0;
}
