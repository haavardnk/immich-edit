import { editsToManifest } from '#lib/edits/manifest.ts';
import type { EditorTab } from '#lib/stores/ui.svelte.ts';
import { lensBlurActive, type Edits } from '#lib/types/edits.ts';

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
  bw: ['bw'],
  lut: ['lut_3d'],
  detail: ['capture_sharpen', 'sharpen', 'ai_denoise', 'luma_nr', 'color_nr'],
  lens: ['lens_profile'],
  'lens-blur': ['lens_blur'],
  effects: ['vignette', 'grain']
};

export function modifiedDevelopPanels(edits: Edits): Set<string> {
  const active = new Set(Object.keys(editsToManifest(edits).ops));
  if (!lensBlurActive(edits.lens_blur)) active.delete('lens_blur');
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
