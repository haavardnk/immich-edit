<script lang="ts">
  import EditableLabel from '$lib/components/EditableLabel.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { maskedEditsIsZero, type MaskLayer } from '$lib/types/edits';
  import type { MaskCapacity } from '$lib/types/masks';
  import { Button, IconButton } from '@immich/ui';
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
</script>

<div
  class="flex h-7 items-center gap-1 rounded-md border px-1 transition-colors {isActive
    ? 'border-primary/25 bg-primary/10'
    : 'border-transparent hover:border-hairline hover:bg-white/4'}"
>
  <div
    class="h-3 w-1 shrink-0 rounded-full ring-1 ring-white/10"
    style="background-color: {layer.color}"
    title="Overlay colour for this mask"
  ></div>
  <IconButton
    size="tiny"
    variant="ghost"
    color="secondary"
    icon={layer.enabled ? mdiEye : mdiEyeOff}
    title={layer.enabled ? 'Disable layer' : 'Enable layer'}
    aria-label="Toggle layer"
    onclick={(e: MouseEvent) => {
      e.stopPropagation();
      void editor.toggleMaskLayerEnabled(layer.id);
    }}
  />
  {#if editing}
    <div class="flex-1 min-w-0">
      <EditableLabel
        ariaLabel="Mask name"
        round
        commitOnBlur
        bind:value={nameDraft}
        oncommit={commitRename}
        oncancel={() => (editing = false)}
      />
    </div>
  {:else}
    <Button
      type="button"
      size="tiny"
      variant="ghost"
      color="secondary"
      class="flex-1 min-w-0 justify-start {layer.enabled ? '' : 'opacity-50'}"
      title="Double-click to rename"
      ondblclick={(e: MouseEvent) => {
        e.stopPropagation();
        beginRename();
      }}
      onclick={() => {
        editor.setActiveLayer(layer.id);
      }}
    >
      <span class="truncate">{layer.name}</span>
    </Button>
  {/if}
  <span
    class="shrink-0 text-[10px] tabular-nums text-dark/65"
    title="{layer.components.length} shape{layer.components.length === 1 ? '' : 's'} in this mask"
  >
    {layer.components.length}
  </span>
  {#if !maskedEditsIsZero(layer.edits)}
    <span class="h-1.5 w-1.5 shrink-0 rounded-full bg-primary/70" title="This mask has adjustments"
    ></span>
  {/if}
  <IconButton
    size="tiny"
    variant="ghost"
    color={isPreview ? 'primary' : 'secondary'}
    icon={mdiCircleOpacity}
    title={isPreview ? 'Hide the mask overlay' : 'Show this mask over the photo'}
    aria-label="Toggle mask preview"
    aria-pressed={isPreview}
    onclick={(e: MouseEvent) => {
      e.stopPropagation();
      togglePreview();
    }}
  />
  {#if isActive && total > 1}
    <IconButton
      size="tiny"
      variant="ghost"
      color="secondary"
      icon={mdiChevronUp}
      title="Move up. Masks further down the list are applied on top."
      aria-label="Move mask up"
      disabled={index === 0}
      onclick={(e: MouseEvent) => {
        e.stopPropagation();
        void editor.reorderMaskLayer(layer.id, index - 1);
      }}
    />
    <IconButton
      size="tiny"
      variant="ghost"
      color="secondary"
      icon={mdiChevronDown}
      title="Move down. Masks further down the list are applied on top."
      aria-label="Move mask down"
      disabled={index === total - 1}
      onclick={(e: MouseEvent) => {
        e.stopPropagation();
        void editor.reorderMaskLayer(layer.id, index + 1);
      }}
    />
  {/if}
  <IconButton
    size="tiny"
    variant="ghost"
    color="secondary"
    icon={mdiContentCopy}
    title="Duplicate layer"
    aria-label="Duplicate layer"
    disabled={cap.layersFull || cap.totalFull}
    onclick={(e: MouseEvent) => {
      e.stopPropagation();
      void editor.duplicateMaskLayer(layer.id);
    }}
  />
  <IconButton
    size="tiny"
    variant="ghost"
    color="secondary"
    icon={mdiClose}
    title="Delete layer"
    aria-label="Delete layer"
    onclick={(e: MouseEvent) => {
      e.stopPropagation();
      void editor.removeMaskLayer(layer.id);
    }}
  />
</div>
