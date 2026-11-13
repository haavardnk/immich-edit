import type { ReorderMove } from '#lib/gestures/reorder.svelte.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import { maskHover } from '#lib/stores/editor/maskHover.svelte.ts';
import { maskThumbs } from '#lib/stores/editor/maskThumbs.svelte.ts';
import { maskModels } from '#lib/stores/maskModels.svelte.ts';
import type { Edits } from '#lib/types/edits.ts';

export function useMaskPanel(): void {
  $effect(() => {
    void maskModels.load();
  });

  $effect(() => {
    maskThumbs.sync(editor.assetId, $state.snapshot(editor.edits) as Edits, editor.initialised);
  });

  $effect(() => maskHover.clear);
}

export function applyReorder({ group, id, to }: ReorderMove): void {
  if (group === 'layers') void editor.reorderMaskLayer(id, to);
  else void editor.reorderMaskComponent(group, id, to);
}
