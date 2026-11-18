import { restoreBrowse } from '#lib/browse/restore.ts';
import { warmNeighbours } from '#lib/editor/neighbours.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import { returnPath } from '#lib/utils/route.ts';
import { onDestroy, onMount, untrack } from 'svelte';
import { on } from 'svelte/events';

function guardPendingSave(event: BeforeUnloadEvent): void {
  if (!editor.saving) return;
  event.preventDefault();
  event.returnValue = '';
}

export function runEditorRoute(id: () => string): void {
  $effect(() => {
    const current = id();
    void untrack(() => editor.load(current));
  });

  let warmedFor: string | null = null;

  $effect(() => {
    const current = editor.assetId;
    const shown = editor.previewUrl !== null || editor.previewFrame !== null;
    if (!current || !shown || current === warmedFor) return;
    if (warmNeighbours(current)) warmedFor = current;
  });

  $effect(() => on(window, 'beforeunload', guardPendingSave));

  onMount(() => {
    if (browsing.assets.length > 0) return;
    void restoreBrowse(returnPath(), id());
  });

  onDestroy(() => {
    void editor.endGeometrySession().finally(() => editor.unload());
  });
}
