import { editor } from '#lib/stores/editor.svelte.ts';
import { ui } from '#lib/stores/ui.svelte.ts';
import type { Edits, GeometryEdits } from '#lib/types/edits.ts';
import { untrack } from 'svelte';

export function useGeometrySession(): void {
  $effect(() => {
    const assetId = editor.assetId;
    const initialised = editor.initialised;
    if (!assetId || !initialised) return;
    untrack(() => editor.startGeometrySession());
    return () => {
      ui.geometryTool = null;
      untrack(() => void editor.endGeometrySession());
    };
  });

  $effect(() => {
    const geometry = $state.snapshot(editor.edits.geometry) as GeometryEdits;
    untrack(() => editor.syncGeometrySession(geometry));
  });

  $effect(() => {
    const edits = $state.snapshot(editor.edits) as Edits;
    untrack(() => editor.refreshGeometryFrame(edits));
  });
}
