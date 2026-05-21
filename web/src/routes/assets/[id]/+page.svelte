<script lang="ts">
  import { page } from '$app/state';
  import { onDestroy, untrack } from 'svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import Viewer from '$lib/components/editor/Viewer.svelte';
  import ImageToolbar from '$lib/components/editor/ImageToolbar.svelte';
  import BottomBar from '$lib/components/editor/BottomBar.svelte';

  const id = $derived(page.params.id as string);

  $effect(() => {
    const current = id;
    untrack(() => editor.load(current));
  });

  onDestroy(() => {
    editor.unload();
  });

  function onKeyDown(e: KeyboardEvent): void {
    const meta = e.metaKey || e.ctrlKey;
    if (meta && e.shiftKey && e.key === 'z') {
      e.preventDefault();
      editor.redo();
    } else if (meta && e.key === 'z') {
      e.preventDefault();
      editor.undo();
    } else if (e.key === 'f' && !meta && !e.shiftKey && !e.altKey) {
      const tag = (e.target as HTMLElement)?.tagName;
      if (tag === 'INPUT' || tag === 'TEXTAREA') return;
      e.preventDefault();
      ui.toggleFullscreen();
    } else if (e.key === '\\' && !meta) {
      e.preventDefault();
      if (!editor.showingOriginal) {
        editor.showingOriginal = true;
        editor.showOriginal();
      }
    }
  }

  function onKeyUp(e: KeyboardEvent): void {
    if (e.key === '\\' && editor.showingOriginal) {
      editor.showingOriginal = false;
      editor.onLive();
    }
  }
</script>

<svelte:window onkeydown={onKeyDown} onkeyup={onKeyUp} />

{#if editor.error}
  <div class="px-4 py-2 text-xs text-red-400 bg-red-400/10 border-b border-red-400/20">{editor.error}</div>
{/if}
<ImageToolbar />
<Viewer />
<BottomBar />
