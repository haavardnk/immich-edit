import { touchUi } from '#lib/stores/touchUi.svelte.ts';
import type { EditorTab } from '#lib/stores/ui.svelte.ts';
import { ui } from '#lib/stores/ui.svelte.ts';
import {
  mdiBandage,
  mdiCropRotate,
  mdiLayersTripleOutline,
  mdiTuneVariant,
  mdiViewGridOutline
} from '@mdi/js';

export type TouchMode = 'presets' | EditorTab;

export type TouchModeDef = { id: TouchMode; label: string; icon: string };

export const TOUCH_MODES: readonly TouchModeDef[] = [
  { id: 'presets', label: 'Presets', icon: mdiViewGridOutline },
  { id: 'develop', label: 'Develop', icon: mdiTuneVariant },
  { id: 'geometry', label: 'Crop', icon: mdiCropRotate },
  { id: 'masks', label: 'Masks', icon: mdiLayersTripleOutline },
  { id: 'retouch', label: 'Retouch', icon: mdiBandage }
];

export function touchMode(): TouchMode | '' {
  if (ui.rightCollapsed) return '';
  return ui.presetStrip ? 'presets' : ui.editorTab;
}

export function openTouchMode(mode: TouchMode): void {
  touchUi.raise();
  if (mode === 'presets') {
    ui.presetStrip = true;
    ui.rightCollapsed = false;
    return;
  }
  ui.presetStrip = false;
  ui.openTab(mode);
}

export function retapTouchMode(event: MouseEvent, mode: TouchMode): void {
  if (touchMode() !== mode) return;
  event.preventDefault();
  ui.togglePanels();
}
