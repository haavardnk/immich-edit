<script lang="ts">
  import ToolbarButton from '$lib/components/ToolbarButton.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import ExifSummary from './ExifSummary.svelte';
  import SoftProofControl from './SoftProofControl.svelte';
  import { hint } from '$lib/keybinds';
  import { copyIndex, isCopy } from '$lib/assetKey';
  import { createVirtualCopy } from '$lib/copies';
  import {
    mdiArrowLeft,
    mdiUndo,
    mdiRedo,
    mdiEyeOutline,
    mdiCompare,
    mdiContentDuplicate,
    mdiTriangleOutline
  } from '@mdi/js';

  const assetId = $derived(editor.assetId);
  const copyBadge = $derived(
    assetId && isCopy(assetId) ? (editor.asset?.copyLabel ?? `Copy ${copyIndex(assetId)}`) : null
  );

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

<div
  class="relative z-30 grid grid-cols-[1fr_auto_1fr] items-center px-3 py-1.5 bg-immich-dark-bg/80 backdrop-blur-sm border-b border-white/5"
>
  <div class="flex items-center gap-1 justify-self-start min-w-0">
    <ToolbarButton
      path={mdiArrowLeft}
      size={18}
      title={hint('Back', 'backToGrid')}
      onclick={goBack}
    />
    {#if editor.asset}
      <span class="text-[13px] font-medium truncate text-immich-dark-fg/80"
        >{editor.asset.originalFileName}</span
      >
    {/if}
    {#if copyBadge}
      <span
        class="shrink-0 px-1.5 py-0.5 rounded bg-white/10 text-[11px] text-immich-dark-fg/70 truncate max-w-40"
        >{copyBadge}</span
      >
    {/if}
  </div>

  <div class="flex items-center gap-0.5 justify-self-center">
    <ToolbarButton
      path={mdiUndo}
      size={18}
      title={hint('Undo', 'undo')}
      disabled={!editor.canUndo}
      onclick={editor.undo}
    />
    <ToolbarButton
      path={mdiRedo}
      size={18}
      title={hint('Redo', 'redo')}
      disabled={!editor.canRedo}
      onclick={editor.redo}
    />
  </div>

  <div class="flex items-center gap-0.5 justify-self-end">
    <ToolbarButton
      path={mdiContentDuplicate}
      size={18}
      title={hint('Create a virtual copy', 'createVirtualCopy')}
      disabled={!assetId}
      onclick={() => assetId && void createVirtualCopy(assetId)}
    />
    <ToolbarButton
      path={mdiEyeOutline}
      size={18}
      title={hint('View original', 'holdOriginal')}
      ariaLabel="View original"
      onpointerdown={() => holdOriginal(true)}
      onpointerup={() => holdOriginal(false)}
      onpointerleave={() => {
        if (editor.showingOriginal) holdOriginal(false);
      }}
    />
    <ToolbarButton
      path={mdiCompare}
      size={18}
      title={hint('Before/after split', 'beforeAfter')}
      ariaLabel="Before/After split"
      active={editor.splitMode}
      disabled={!!editor.geometrySession}
      onclick={editor.toggleSplit}
    />
    <ToolbarButton
      path={mdiTriangleOutline}
      size={18}
      title={hint('Clipping overlay', 'clipWarn')}
      ariaLabel="Clipping overlay"
      active={ui.clipWarn}
      pressed={ui.clipWarn}
      onclick={editor.toggleClipWarn}
    />
    <SoftProofControl />
    {#if editor.assetId}
      <ExifSummary />
    {/if}
  </div>
</div>
