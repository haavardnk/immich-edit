import { editor } from '#lib/stores/editor.svelte.ts';
import { ui } from '#lib/stores/ui.svelte.ts';
import { nudgeable } from '#lib/utils/maskDrag.ts';
import { page } from '$app/state';

export function editorAssetId(): string | null {
  return page.params.id ?? null;
}

export function returnPath(): string | null {
  return page.url.searchParams.get('from');
}

export function selectedShapeNudges(): boolean {
  if (ui.editorTab !== 'masks' || !editor.activeMaskComponentId) return false;
  const layer = editor.edits.masks.find((l) => l.id === editor.activeLayerId);
  const comp = layer?.components.find((c) => c.id === editor.activeMaskComponentId);
  return !!comp && nudgeable(comp.kind);
}
