import { editor } from '#lib/stores/editor.svelte.ts';
import { ui } from '#lib/stores/ui.svelte.ts';
import { nudgeable } from '#lib/utils/maskDrag.ts';
import { page } from '$app/state';
import { backToGrid } from '#lib/browse/backToGrid.ts';
import { returnPath } from '#lib/utils/route.ts';

export function editorAssetId(): string | null {
  return page.params.id ?? null;
}

export function leaveEditor(id: string | null): void {
  void backToGrid(id, returnPath());
}

export function selectedShapeNudges(): boolean {
  if (ui.editorTab !== 'masks' || !editor.activeMaskComponentId) return false;
  const layer = editor.edits.masks.find((l) => l.id === editor.activeLayerId);
  const comp = layer?.components.find((c) => c.id === editor.activeMaskComponentId);
  return !!comp && nudgeable(comp.kind);
}
