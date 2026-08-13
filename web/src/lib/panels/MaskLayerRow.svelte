<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { maskedEditsIsZero, type MaskLayer } from '$lib/types/edits';
  import type { MaskCapacity } from '$lib/types/masks';
  import {
    mdiChevronDown,
    mdiChevronUp,
    mdiCircleOpacity,
    mdiClose,
    mdiContentCopy,
    mdiEye,
    mdiEyeOff
  } from '@mdi/js';

  let {
    layer,
    index,
    total,
    cap
  }: {
    layer: MaskLayer;
    index: number;
    total: number;
    cap: MaskCapacity;
  } = $props();

  let editing = $state(false);
  let nameDraft = $state('');

  const isActive = $derived(editor.activeLayerId === layer.id);
  const isPreview = $derived(editor.maskPreviewLayerId === layer.id);

  function beginRename(): void {
    nameDraft = layer.name;
    editing = true;
  }

  async function commitRename(): Promise<void> {
    const next = nameDraft.trim();
    editing = false;
    if (next && next !== layer.name) await editor.renameMaskLayer(layer.id, next);
  }

  function togglePreview(): void {
    if (isPreview) editor.endMaskPreview();
    else editor.previewMaskWeight(layer.id);
  }

  function focusOnMount(node: HTMLInputElement): void {
    node.focus();
    node.select();
  }
</script>

<div
  class="flex items-center gap-1.5 px-1.5 py-1 rounded transition-colors cursor-pointer {isActive
    ? 'bg-white/10'
    : 'hover:bg-white/5'}"
  role="button"
  tabindex="0"
  onclick={() => editor.setActiveLayer(layer.id)}
  onkeydown={(e) => {
    if (e.key === 'Enter' || e.key === ' ') editor.setActiveLayer(layer.id);
  }}
>
  <div
    class="w-3 h-3 rounded-sm ring-1 ring-white/20 shrink-0"
    style="background-color: {layer.color}"
    title="Overlay colour for this mask"
  ></div>
  <button
    type="button"
    class="shrink-0 text-immich-dark-fg/50 hover:text-immich-dark-fg"
    title={layer.enabled ? 'Disable layer' : 'Enable layer'}
    aria-label="Toggle layer"
    onclick={(e) => {
      e.stopPropagation();
      void editor.toggleMaskLayerEnabled(layer.id);
    }}
  >
    <Icon path={layer.enabled ? mdiEye : mdiEyeOff} size={13} />
  </button>
  {#if editing}
    <input
      class="flex-1 bg-white/5 border border-white/10 rounded px-1 text-xs text-immich-dark-fg outline-none"
      bind:value={nameDraft}
      onblur={() => void commitRename()}
      onkeydown={(e) => {
        if (e.key === 'Enter') (e.currentTarget as HTMLInputElement).blur();
        else if (e.key === 'Escape') editing = false;
      }}
      use:focusOnMount
      onclick={(e) => e.stopPropagation()}
    />
  {:else}
    <button
      type="button"
      class="flex-1 text-left text-xs text-immich-dark-fg/90 truncate {layer.enabled
        ? ''
        : 'opacity-50'}"
      ondblclick={(e) => {
        e.stopPropagation();
        beginRename();
      }}
      onclick={(e) => {
        e.stopPropagation();
        editor.setActiveLayer(layer.id);
      }}
      title="Double-click to rename"
    >
      {layer.name}
    </button>
  {/if}
  <span
    class="shrink-0 text-[10px] tabular-nums text-immich-dark-fg/30"
    title="{layer.components.length} shape{layer.components.length === 1 ? '' : 's'} in this mask"
  >
    {layer.components.length}
  </span>
  {#if !maskedEditsIsZero(layer.edits)}
    <span
      class="shrink-0 w-1.5 h-1.5 rounded-full bg-immich-dark-primary/70"
      title="This mask has adjustments"
    ></span>
  {/if}
  <button
    type="button"
    class="shrink-0 text-immich-dark-fg/40 hover:text-immich-dark-fg transition-colors {isPreview
      ? 'text-immich-dark-primary'
      : ''}"
    title={isPreview ? 'Hide the mask overlay' : 'Show this mask over the photo'}
    aria-label="Toggle mask preview"
    onclick={(e) => {
      e.stopPropagation();
      togglePreview();
    }}
  >
    <Icon path={mdiCircleOpacity} size={13} />
  </button>
  {#if isActive && total > 1}
    <button
      type="button"
      class="shrink-0 text-immich-dark-fg/40 hover:text-immich-dark-fg transition-colors disabled:opacity-20 disabled:cursor-not-allowed"
      title="Move up. Masks further down the list are applied on top."
      aria-label="Move mask up"
      disabled={index === 0}
      onclick={(e) => {
        e.stopPropagation();
        void editor.reorderMaskLayer(layer.id, index - 1);
      }}
    >
      <Icon path={mdiChevronUp} size={13} />
    </button>
    <button
      type="button"
      class="shrink-0 text-immich-dark-fg/40 hover:text-immich-dark-fg transition-colors disabled:opacity-20 disabled:cursor-not-allowed"
      title="Move down. Masks further down the list are applied on top."
      aria-label="Move mask down"
      disabled={index === total - 1}
      onclick={(e) => {
        e.stopPropagation();
        void editor.reorderMaskLayer(layer.id, index + 1);
      }}
    >
      <Icon path={mdiChevronDown} size={13} />
    </button>
  {/if}
  <button
    type="button"
    class="shrink-0 text-immich-dark-fg/40 hover:text-immich-dark-fg transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
    title="Duplicate layer"
    aria-label="Duplicate layer"
    disabled={cap.layersFull || cap.totalFull}
    onclick={(e) => {
      e.stopPropagation();
      void editor.duplicateMaskLayer(layer.id);
    }}
  >
    <Icon path={mdiContentCopy} size={12} />
  </button>
  <button
    type="button"
    class="shrink-0 text-immich-dark-fg/40 hover:text-red-400 transition-colors"
    title="Delete layer"
    aria-label="Delete layer"
    onclick={(e) => {
      e.stopPropagation();
      void editor.removeMaskLayer(layer.id);
    }}
  >
    <Icon path={mdiClose} size={13} />
  </button>
</div>
