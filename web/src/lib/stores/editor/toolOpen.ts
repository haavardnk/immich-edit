import { layout } from '#lib/device/layout.svelte.ts';
import { ui, type EditorTab } from '#lib/stores/ui.svelte.ts';

export function toolOpen(tab: EditorTab): boolean {
  if (ui.editorTab !== tab) return false;
  return layout.mode !== 'touch' || !ui.presetStrip;
}
