<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { onDestroy, untrack } from 'svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { browsing } from '$lib/stores/browsing.svelte';
  import { nextRatingFromKey } from '$lib/ratingShortcuts';
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
    if (e.key === 'Escape') {
      if (ui.closeMetadataPopovers()) {
        e.preventDefault();
        return;
      }
      if (isTypingTarget(e)) return;
      if (ui.keybindsHelpOpen) {
        e.preventDefault();
        ui.closeKeybindsHelp();
      } else if (ui.editorTab === 'geometry' && ui.perspectiveCorners) {
        e.preventDefault();
        ui.perspectiveCorners = false;
      } else if (ui.editorTab === 'geometry' && !ui.fullscreen) {
        e.preventDefault();
        ui.editorTab = 'develop';
      } else if (ui.editorTab === 'retouch' && editor.activeRetouchId) {
        e.preventDefault();
        editor.activeRetouchId = null;
      } else if (editor.activeMaskComponentId) {
        e.preventDefault();
        editor.setActiveMaskComponent(null);
      } else if (ui.fullscreen) {
        e.preventDefault();
        ui.toggleFullscreen();
      }
      return;
    }
    if (isTypingTarget(e)) return;
    if (e.key === '?' || (e.key === '/' && e.shiftKey)) {
      e.preventDefault();
      ui.toggleKeybindsHelp();
      return;
    }
    if (ui.keybindsHelpOpen) return;
    if ((e.key === 'ArrowLeft' || e.key === 'j' || e.key === 'J') && !meta && !e.altKey) {
      const prev = browsing.prevOf(id);
      if (!prev) return;
      e.preventDefault();
      void goto(`/assets/${prev.id}`, { replaceState: true });
      return;
    }
    if ((e.key === 'ArrowRight' || e.key === 'k' || e.key === 'K') && !meta && !e.altKey) {
      const next = browsing.nextOf(id);
      if (!next) return;
      e.preventDefault();
      void goto(`/assets/${next.id}`, { replaceState: true });
      return;
    }
    if ((e.key === ' ' || e.key === 'z' || e.key === 'Z') && !meta && !e.shiftKey && !e.altKey) {
      e.preventDefault();
      ui.zoomToggle();
      return;
    }
    if (e.key === 'i' && !meta && !e.shiftKey && !e.altKey) {
      e.preventDefault();
      ui.toggleExifPopover();
      return;
    }
    if (e.key === 't' && !meta && !e.shiftKey && !e.altKey) {
      e.preventDefault();
      ui.toggleTagsPopover();
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
    if ((e.key === 'x' || e.key === 'X') && !meta && !e.altKey) {
      e.preventDefault();
      void editor.toggleReject();
      return;
    }
    if (!meta && !e.shiftKey && !e.altKey) {
      const next = nextRatingFromKey(e.key, editor.asset?.exifInfo?.rating ?? null);
      if (next !== undefined) {
        e.preventDefault();
        void editor.setRating(next);
        return;
      }
    }
    if (e.key === '\\' && !meta) {
      e.preventDefault();
      if (!editor.showingOriginal) {
        editor.showingOriginal = true;
        editor.showOriginal();
      }
    }
    if ((e.key === 'b' || e.key === 'B') && !meta && !e.shiftKey && !e.altKey) {
      e.preventDefault();
      if (editor.showingOriginal) {
        editor.showingOriginal = false;
        editor.onLive();
      } else {
        editor.showingOriginal = true;
        editor.showOriginal();
      }
      return;
    }
    if ((e.key === 'o' || e.key === 'O') && !meta && !e.shiftKey && !e.altKey) {
      e.preventDefault();
      editor.toggleClipWarn();
      return;
    }
    if ((e.key === 'g' || e.key === 'G') && !meta && !e.shiftKey && !e.altKey) {
      e.preventDefault();
      ui.openGeometry();
      return;
    }
    if ((e.key === 'p' || e.key === 'P') && !meta && !e.shiftKey && !e.altKey) {
      e.preventDefault();
      if (ui.editorTab !== 'geometry') ui.openGeometry();
      ui.togglePerspectiveCorners();
      return;
    }
    if ((e.key === 'r' || e.key === 'R') && !meta && !e.shiftKey && !e.altKey) {
      e.preventDefault();
      void editor.onReset();
      return;
    }
    if ((e.key === 'v' || e.key === 'V') && !meta && !e.shiftKey && !e.altKey) {
      e.preventDefault();
      ui.openRetouch();
      return;
    }
    if (ui.editorTab === 'retouch' && !meta && !e.shiftKey && !e.altKey) {
      if (e.key === 'h' || e.key === 'H') {
        e.preventDefault();
        editor.setRetouchMode('heal');
        return;
      }
      if (e.key === 'c' || e.key === 'C') {
        e.preventDefault();
        editor.setRetouchMode('clone');
        return;
      }
      if (e.key === '[' || e.key === ']') {
        e.preventDefault();
        const step = e.key === '[' ? -0.005 : 0.005;
        editor.setRetouchTool({
          size: Math.min(0.3, Math.max(0.005, editor.retouchTool.size + step))
        });
        return;
      }
      if ((e.key === 'Delete' || e.key === 'Backspace') && editor.activeRetouchId) {
        e.preventDefault();
        void editor.removeRetouchStroke(editor.activeRetouchId);
        return;
      }
    }
  }

  function onKeyUp(e: KeyboardEvent): void {
    if (e.key === '\\' && editor.showingOriginal) {
      editor.showingOriginal = false;
      editor.onLive();
    }
  }

  let viewportWidth = $state(typeof window !== 'undefined' ? window.innerWidth : 1920);
  const tooNarrow = $derived(viewportWidth < 768);
</script>

<svelte:window onkeydown={onKeyDown} onkeyup={onKeyUp} onresize={() => (viewportWidth = window.innerWidth)} />

{#if tooNarrow}
  <div class="flex-1 flex items-center justify-center p-6 text-center">
    <div class="max-w-sm space-y-2">
      <h2 class="text-sm font-medium text-immich-dark-fg">Desktop required</h2>
      <p class="text-xs text-immich-dark-fg/60">
        immich-edit requires a desktop browser (≥ 768px) for editing. Please switch to a larger screen.
      </p>
    </div>
  </div>
{:else}
  {#if editor.error}
    <div class="px-4 py-2 text-xs text-red-400 bg-red-400/10 border-b border-red-400/20">{editor.error}</div>
  {/if}
  <ImageToolbar />
  <Viewer />
  <BottomBar />
{/if}
