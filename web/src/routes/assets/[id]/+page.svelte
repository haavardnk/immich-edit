<script lang="ts">
  import { page } from '$app/state';
  import { onDestroy, untrack } from 'svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { editorKeydown, editorKeyup } from '$lib/keymaps/editor';
  import Viewer from '$lib/components/editor/Viewer.svelte';
  import ImageToolbar from '$lib/components/editor/ImageToolbar.svelte';
  import BottomBar from '$lib/components/editor/BottomBar.svelte';

  const id = $derived(page.params.id as string);

  $effect(() => {
    const current = id;
    untrack(() => editor.load(current));
  });

  onDestroy(() => {
    void editor.finishGeometrySession().finally(() => editor.unload());
  });

  let viewportWidth = $state(typeof window !== 'undefined' ? window.innerWidth : 1920);
  const tooNarrow = $derived(viewportWidth < 768);
</script>

<svelte:window
  onkeydown={(e) => editorKeydown(e, id)}
  onkeyup={editorKeyup}
  onresize={() => (viewportWidth = window.innerWidth)}
/>

{#if tooNarrow}
  <div class="flex-1 flex items-center justify-center p-6 text-center">
    <div class="max-w-sm space-y-2">
      <h2 class="text-sm font-medium text-immich-dark-fg">Desktop required</h2>
      <p class="text-xs text-immich-dark-fg/60">
        immich-edit requires a desktop browser (≥ 768px) for editing. Please switch to a larger
        screen.
      </p>
    </div>
  </div>
{:else}
  {#if editor.error}
    <div class="px-4 py-2 text-xs text-red-400 bg-red-400/10 border-b border-red-400/20">
      {editor.error}
    </div>
  {/if}
  <ImageToolbar />
  <Viewer />
  <BottomBar />
{/if}
