<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { onDestroy, untrack } from 'svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { browsing } from '$lib/stores/browsing.svelte';
  import { createVirtualCopy } from '$lib/copies';
  import { nextRatingFromKey } from '$lib/ratingShortcuts';
  import { isKeybind, isTypingTarget, matchKeybind } from '$lib/keybinds';
  import { activeContexts } from '$lib/keybindContext';
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

  function onEscape(e: KeyboardEvent): void {
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
  }

  function stepBrush(key: string, patch: (delta: number) => void): void {
    patch(key === '[' || key === '{' ? -1 : 1);
  }

  function onKeyDown(e: KeyboardEvent): void {
    if (isKeybind(e, 'editorEscape')) {
      onEscape(e);
      return;
    }
    if (isTypingTarget(e)) return;
    if (isKeybind(e, 'help')) {
      e.preventDefault();
      ui.toggleKeybindsHelp();
      return;
    }
    if (ui.keybindsHelpOpen) return;

    const bind = matchKeybind(e, activeContexts());
    if (!bind || bind === 'maskDelete' || bind === 'maskClosePolygon') return;
    e.preventDefault();

    switch (bind) {
      case 'editorNav': {
        const target = e.key === 'ArrowLeft' ? browsing.prevOf(id) : browsing.nextOf(id);
        if (target) void goto(`/assets/${target.id}`, { replaceState: true });
        return;
      }
      case 'backToGrid':
        if (window.history.length > 1) window.history.back();
        return;
      case 'undo':
        editor.undo();
        return;
      case 'redo':
        editor.redo();
        return;
      case 'zoomToggle':
        ui.zoomToggle();
        return;
      case 'toggleInfo':
        ui.toggleExifPopover();
        return;
      case 'toggleTags':
        ui.toggleTagsPopover();
        return;
      case 'clipWarn':
        editor.toggleClipWarn();
        return;
      case 'beforeAfter':
        editor.toggleSplit();
        return;
      case 'holdOriginal':
        if (!editor.showingOriginal) {
          editor.showingOriginal = true;
          editor.showOriginal();
        }
        return;
      case 'togglePanels':
        ui.togglePanels();
        return;
      case 'toggleChrome':
        ui.toggleChrome();
        return;
      case 'fullscreen':
        ui.toggleFullscreen();
        return;
      case 'openGeometry':
        ui.openGeometry();
        return;
      case 'openRetouch':
        ui.openRetouch();
        return;
      case 'openMasks':
        ui.openMasks();
        return;
      case 'openExport':
        ui.openExport();
        return;
      case 'createVirtualCopy':
        void createVirtualCopy(id);
        return;
      case 'perspective':
        if (ui.editorTab !== 'geometry') ui.openGeometry();
        ui.togglePerspectiveCorners();
        return;
      case 'resetEdits':
        void editor.onReset();
        return;
      case 'copyEdits':
        editor.copyEdits();
        return;
      case 'pasteEdits':
        void editor.pasteEdits();
        return;
      case 'favorite':
        void editor.toggleFavorite();
        return;
      case 'reject':
        void editor.toggleReject();
        return;
      case 'unflag':
        void editor.clearFlags();
        return;
      case 'rate': {
        const next = nextRatingFromKey(e.key, editor.asset?.exifInfo?.rating ?? null);
        if (next !== undefined) void editor.setRating(next);
        return;
      }
      case 'maskOverlay':
        editor.toggleMaskOverlay();
        return;
      case 'retouchHeal':
        editor.setRetouchMode('heal');
        return;
      case 'retouchClone':
        editor.setRetouchMode('clone');
        return;
      case 'retouchDelete':
        if (editor.activeRetouchId) void editor.removeRetouchStroke(editor.activeRetouchId);
        return;
      case 'brushSize':
        if (ui.editorTab === 'retouch') {
          stepBrush(e.key, (d) =>
            editor.setRetouchTool({
              size: Math.min(0.3, Math.max(0.005, editor.retouchTool.size + d * 0.005))
            })
          );
        } else {
          stepBrush(e.key, (d) =>
            editor.setBrushTool({
              size: Math.min(0.5, Math.max(0.005, editor.brushTool.size + d * 0.01))
            })
          );
        }
        return;
      case 'brushHardness':
        if (ui.editorTab === 'retouch') {
          stepBrush(e.key, (d) =>
            editor.setRetouchTool({
              hardness: Math.min(1, Math.max(0, editor.retouchTool.hardness + d * 0.1))
            })
          );
        } else {
          stepBrush(e.key, (d) =>
            editor.setBrushTool({
              hardness: Math.min(1, Math.max(0, editor.brushTool.hardness + d * 0.1))
            })
          );
        }
        return;
    }
  }

  function onKeyUp(e: KeyboardEvent): void {
    if (isKeybind(e, 'holdOriginal') && editor.showingOriginal) {
      editor.showingOriginal = false;
      editor.onLive();
    }
  }

  let viewportWidth = $state(typeof window !== 'undefined' ? window.innerWidth : 1920);
  const tooNarrow = $derived(viewportWidth < 768);
</script>

<svelte:window
  onkeydown={onKeyDown}
  onkeyup={onKeyUp}
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
