<script lang="ts">
  import ToolbarButton from '$lib/components/ToolbarButton.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import ExifSummary from './ExifSummary.svelte';
  import {
    mdiArrowLeft,
    mdiUndo,
    mdiRedo,
    mdiEyeOutline,
    mdiCompare,
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

<div class="relative z-30 grid grid-cols-[1fr_auto_1fr] items-center px-3 py-1.5 bg-immich-dark-bg/80 backdrop-blur-sm border-b border-white/5">
  <div class="flex items-center gap-1 justify-self-start min-w-0">
    <ToolbarButton path={mdiArrowLeft} size={18} title="Back" onclick={goBack} />
    {#if editor.asset}
      <span class="text-[13px] font-medium truncate text-immich-dark-fg/80">{editor.asset.originalFileName}</span>
    {/if}
  </div>

  <div class="flex items-center gap-0.5 justify-self-center">
    <ToolbarButton
      path={mdiUndo}
      size={18}
      title="Undo (Ctrl+Z)"
      disabled={!editor.canUndo}
      onclick={editor.undo}
    />
    <ToolbarButton
      path={mdiRedo}
      size={18}
      title="Redo (Ctrl+Shift+Z)"
      disabled={!editor.canRedo}
      onclick={editor.redo}
    />
  </div>

  <div class="flex items-center gap-0.5 justify-self-end">
    <ToolbarButton
      path={mdiEyeOutline}
      size={18}
      title="View Original (hold \)"
      onpointerdown={() => holdOriginal(true)}
      onpointerup={() => holdOriginal(false)}
      onpointerleave={() => { if (editor.showingOriginal) holdOriginal(false); }}
    />
    <ToolbarButton
      path={mdiCompare}
      size={18}
      title="Before/After split"
      active={editor.splitMode}
      disabled={!!editor.cropSession}
      onclick={editor.toggleSplit}
    />
    {#if editor.assetId}
      <ExifSummary />
    {/if}
  </div>
</div>
