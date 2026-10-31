<script lang="ts">
  import { page } from '$app/state';
  import { onDestroy, onMount, untrack } from 'svelte';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { browsing } from '#lib/stores/browsing.svelte.ts';
  import { restoreBrowse } from '#lib/browse/restore.ts';
  import { editorKeydown, editorKeyup } from '#lib/shortcuts/editorKeymap.ts';
  import { warmNeighbours } from '#lib/editor/neighbours.ts';
  import Viewer from '#lib/components/editor/Viewer.svelte';
  import ImageToolbar from '#lib/components/editor/ImageToolbar.svelte';
  import BottomBar from '#lib/components/editor/BottomBar.svelte';
  import Notice from '#lib/components/Notice.svelte';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import { hint } from '#lib/shortcuts/labels.ts';
  import { IconButton } from '@immich/ui';
  import { mdiFullscreenExit } from '@mdi/js';
  import { innerWidth } from 'svelte/reactivity/window';

  const id = $derived(page.params.id as string);

  $effect(() => {
    const current = id;
    void untrack(() => editor.load(current));
  });

  let warmedFor: string | null = null;

  $effect(() => {
    const current = editor.assetId;
    const shown = editor.previewUrl !== null || editor.previewFrame !== null;
    if (!current || !shown || current === warmedFor) return;
    if (warmNeighbours(current)) warmedFor = current;
  });

  onMount(() => {
    if (browsing.assets.length > 0) return;
    void restoreBrowse(page.url.searchParams.get('from'), id);
  });

  onDestroy(() => {
    void editor.endGeometrySession().finally(() => editor.unload());
  });

  function guardPendingSave(event: BeforeUnloadEvent): void {
    if (!editor.saving) return;
    event.preventDefault();
    event.returnValue = '';
  }

  const tooNarrow = $derived((innerWidth.current ?? 1920) < 768);
</script>

<svelte:window
  onbeforeunload={guardPendingSave}
  onkeydown={(e) => editorKeydown(e, id)}
  onkeyup={editorKeyup}
/>

{#if tooNarrow}
  <div class="flex-1 flex items-center justify-center p-6 text-center">
    <div class="max-w-sm space-y-2">
      <h2 class="text-sm font-medium text-dark">Desktop required</h2>
      <p class="text-xs text-dark/65">
        immich-edit requires a desktop browser (≥ 768px) for editing. Please switch to a larger
        screen.
      </p>
    </div>
  </div>
{:else}
  {#if editor.error}
    <Notice message={editor.error} class="mx-4 my-2 text-xs" />
  {/if}
  {#if !ui.fullscreen}<ImageToolbar />{/if}
  <Viewer />
  {#if ui.fullscreen}
    <IconButton
      size="small"
      variant="filled"
      color="primary"
      shape="round"
      class="fixed top-3 right-3 z-40 shadow-lg"
      icon={mdiFullscreenExit}
      title={hint('Exit fullscreen', 'fullscreen')}
      aria-label={hint('Exit fullscreen', 'fullscreen')}
      onclick={ui.toggleFullscreen}
    />
  {:else}
    <BottomBar />
  {/if}
{/if}
