<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import ExifSummary from './ExifSummary.svelte';
  import {
    mdiArrowLeft,
    mdiUndo,
    mdiRedo,
    mdiEyeOutline,
    mdiFullscreen,
    mdiFullscreenExit,
  } from '@mdi/js';

  function goBack(): void {
    if (window.history.length > 1) {
      window.history.back();
    }
  }

  function holdOriginal(down: boolean): void {
    editor.showingOriginal = down;
    if (down) {
      editor.showOriginal();
    } else {
      editor.onLive();
    }
  }
</script>

<div class="flex items-center justify-between px-3 py-1.5 bg-immich-dark-bg/80 backdrop-blur-sm border-b border-white/5">
  <div class="flex items-center gap-1">
    <button
      class="btn btn-ghost btn-sm btn-square"
      title="Back"
      onclick={goBack}
    >
      <Icon path={mdiArrowLeft} size={20} />
    </button>
  </div>

  <div class="flex items-center gap-1">
    <button
      class="btn btn-ghost btn-sm btn-square {editor.canUndo ? 'text-immich-dark-fg' : 'text-immich-dark-fg/25 cursor-not-allowed'}"
      title="Undo (Ctrl+Z)"
      disabled={!editor.canUndo}
      onclick={editor.undo}
    >
      <Icon path={mdiUndo} size={20} />
    </button>
    <button
      class="btn btn-ghost btn-sm btn-square {editor.canRedo ? 'text-immich-dark-fg' : 'text-immich-dark-fg/25 cursor-not-allowed'}"
      title="Redo (Ctrl+Shift+Z)"
      disabled={!editor.canRedo}
      onclick={editor.redo}
    >
      <Icon path={mdiRedo} size={20} />
    </button>
  </div>

  <div class="flex items-center gap-1">
    <button
      class="btn btn-ghost btn-sm btn-square"
      title="View Original (hold \)"
      onpointerdown={() => holdOriginal(true)}
      onpointerup={() => holdOriginal(false)}
      onpointerleave={() => { if (editor.showingOriginal) holdOriginal(false); }}
    >
      <Icon path={mdiEyeOutline} size={20} />
    </button>
    <button
      class="btn btn-ghost btn-sm btn-square"
      title="Fullscreen (⇧F)"
      onclick={ui.toggleFullscreen}
    >
      <Icon path={ui.fullscreen ? mdiFullscreenExit : mdiFullscreen} size={20} />
    </button>
    {#if editor.assetId}
      <ExifSummary />
    {/if}
  </div>
</div>
