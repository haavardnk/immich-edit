<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { onDestroy, untrack } from 'svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { browsing } from '$lib/stores/browsing.svelte';
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

  function isTypingTarget(e: KeyboardEvent): boolean {
    const el = e.target as HTMLElement | null;
    if (!el) return false;
    const tag = el.tagName;
    if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return true;
    return el.isContentEditable;
  }

  function onKeyDown(e: KeyboardEvent): void {
    const meta = e.metaKey || e.ctrlKey;
    if (meta && e.shiftKey && e.key === 'z') {
      e.preventDefault();
      editor.redo();
      return;
    }
    if (meta && e.key === 'z') {
      e.preventDefault();
      editor.undo();
      return;
    }
    if (isTypingTarget(e)) return;
    if (e.key === 'Escape') {
      if (ui.keybindsHelpOpen) {
        e.preventDefault();
        ui.closeKeybindsHelp();
      } else if (ui.fullscreen) {
        e.preventDefault();
        ui.toggleFullscreen();
      }
      return;
    }
    if (e.key === '?' || (e.key === '/' && e.shiftKey)) {
      e.preventDefault();
      ui.toggleKeybindsHelp();
      return;
    }
    if (ui.keybindsHelpOpen) return;
    if (e.key === 'ArrowLeft' && !meta && !e.altKey) {
      if (editor.cropSession) return;
      const prev = browsing.prevOf(id);
      if (!prev) return;
      e.preventDefault();
      void goto(`/assets/${prev.id}`);
      return;
    }
    if (e.key === 'ArrowRight' && !meta && !e.altKey) {
      if (editor.cropSession) return;
      const next = browsing.nextOf(id);
      if (!next) return;
      e.preventDefault();
      void goto(`/assets/${next.id}`);
      return;
    }
    if (e.key === ' ' && !meta && !e.shiftKey && !e.altKey) {
      e.preventDefault();
      ui.zoomToggle();
      return;
    }
    if (e.key === '0' && !meta && !e.shiftKey && !e.altKey) {
      e.preventDefault();
      ui.zoomFit();
      return;
    }
    if (e.key === 'F' && !meta && e.shiftKey && !e.altKey) {
      e.preventDefault();
      ui.toggleFullscreen();
      return;
    }
    if (e.key === 'f' && !meta && !e.shiftKey && !e.altKey) {
      e.preventDefault();
      void editor.toggleFavorite();
      return;
    }
    if (!meta && !e.shiftKey && !e.altKey && e.key >= '1' && e.key <= '5') {
      e.preventDefault();
      const n = Number(e.key);
      const current = editor.asset?.exifInfo?.rating ?? null;
      void editor.setRating(current === n ? null : n);
      return;
    }
    if (e.key === '\\' && !meta) {
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
