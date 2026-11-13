<script lang="ts">
  import EditableLabel from '#lib/components/EditableLabel.svelte';
  import ElementTooltip from '#lib/components/ElementTooltip.svelte';
  import type { Reorder } from '#lib/gestures/reorder.svelte.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { maskHover } from '#lib/stores/editor/maskHover.svelte.ts';
  import { maskThumbs } from '#lib/stores/editor/maskThumbs.svelte.ts';
  import { maskedEditsIsZero, type MaskLayer } from '#lib/types/edits.ts';
  import type { MaskCapacity } from '#lib/types/masks.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { Button, Icon, IconButton } from '@immich/ui';
  import { mergeProps } from 'bits-ui';
  import {
    mdiChevronDown,
    mdiChevronUp,
    mdiCircleOpacity,
    mdiClose,
    mdiContentCopy,
    mdiDragVertical,
    mdiEye,
    mdiEyeOff,
    mdiLayersOutline
  } from '@mdi/js';

  let {
    layer,
    index,
    order,
    reorder,
    cap
  }: {
    layer: MaskLayer;
    index: number;
    order: readonly string[];
    reorder: Reorder;
    cap: MaskCapacity;
  } = $props();

  const total = $derived(order.length);
  const thumb = $derived(maskThumbs.urls[layer.id]);
  const lifted = $derived(reorder.dragging?.id === layer.id);

  let editing = $state(false);
  let nameDraft = $state('');

  const isActive = $derived(editor.activeLayerId === layer.id);
  const overlayShown = $derived(isActive && editor.maskOverlayVisible);

  function beginRename(): void {
    nameDraft = layer.name;
    editing = true;
  }

  async function commitRename(): Promise<void> {
    const next = nameDraft.trim();
    editing = false;
    if (next && next !== layer.name) await editor.renameMaskLayer(layer.id, next);
  }

  function hover(e: PointerEvent): void {
    if (e.pointerType !== 'mouse' || reorder.dragging) return;
    maskHover.enter({ layerId: layer.id, componentId: null, label: layer.name, mode: null });
  }
</script>

<div
  role="listitem"
  data-reorder-item
  data-testid="mask-layer-row"
  style:translate={reorder.active('layers')
    ? `0 ${reorder.offset('layers', order, layer.id)}px`
    : undefined}
  class="flex h-7 items-center gap-1 rounded-md border px-1 {lifted
    ? 'relative z-10 border-hairline bg-light-200 shadow-popover'
    : isActive
      ? 'border-primary/25 bg-primary/10'
      : 'border-transparent hover:border-hairline hover:bg-light-950/4'} {reorder.active(
    'layers'
  ) && !lifted
    ? 'transition-[translate] duration-150'
    : 'transition-colors'}"
  onpointerenter={hover}
  onpointerleave={() => maskHover.leave(layer.id, null)}
>
  <IconButton
    size="tiny"
    variant="ghost"
    color="secondary"
    icon={mdiDragVertical}
    title="Drag or use the arrow keys to reorder. Masks further down are applied on top."
    aria-label="Reorder {layer.name}"
    disabled={total < 2}
    class="-mx-0.5 w-4 shrink-0 cursor-grab touch-none opacity-50 hover:opacity-100"
    {...reorder.handle('layers', order, layer.id)}
  />
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
  {#if thumb}
    <img
      src={thumb}
      alt=""
      class="size-5 shrink-0 rounded-sm bg-image-canvas object-cover ring-1 ring-hairline"
      data-testid="mask-thumb"
    />
  {:else}
    <span
      class="flex size-5 shrink-0 items-center justify-center rounded-sm text-dark/40 ring-1 ring-hairline"
      data-testid="mask-thumb-pending"
    >
      <Icon icon={mdiLayersOutline} size="14px" aria-hidden="true" />
    </span>
  {/if}
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
  <ElementTooltip text="{countOf(layer.components.length, 'shape')} in this mask">
    {#snippet child({ props })}
      <span
        {...mergeProps(props, { tabindex: -1 })}
        class="shrink-0 text-[10px] tabular-nums text-dark/65"
      >
        {countOf(layer.components.length, 'shape')}
      </span>
    {/snippet}
  </ElementTooltip>
  {#if !maskedEditsIsZero(layer.edits)}
    <ElementTooltip text="This mask has adjustments">
      {#snippet child({ props })}
        <span
          {...mergeProps(props, { tabindex: -1 })}
          class="h-1.5 w-1.5 shrink-0 rounded-full bg-primary/70"
        ></span>
      {/snippet}
    </ElementTooltip>
  {/if}
  <IconButton
    size="tiny"
    variant="ghost"
    color={overlayShown ? 'primary' : 'secondary'}
    icon={mdiCircleOpacity}
    title={overlayShown ? 'Hide the mask overlay' : "Show this mask's overlay"}
    aria-label="Toggle mask preview"
    aria-pressed={overlayShown}
    onclick={(e: MouseEvent) => {
      e.stopPropagation();
      editor.toggleLayerOverlay(layer.id);
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
