<script lang="ts">
  import { currentPath } from '#lib/utils/route.ts';
  import DeleteConfirmation from '#lib/components/DeleteConfirmation.svelte';
  import IconLink from '#lib/components/IconLink.svelte';
  import TileBand from '#lib/components/browse/tile/TileBand.svelte';
  import TileMarks from '#lib/components/browse/tile/TileMarks.svelte';
  import { ownsAsset } from '#lib/browse/ownership.ts';
  import type { AssetSummary } from '#lib/types/album.ts';
  import { hint } from '#lib/shortcuts/labels.ts';
  import { assetPlaceholder, assetThumbUrl } from '#lib/api/assets.ts';
  import { isCopy } from '#lib/browse/assetKey.ts';
  import { isRejected } from '#lib/browse/reject.ts';
  import { tileDrag } from '#lib/browse/tileDrag.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { labelOf } from '#lib/stores/labels.ts';
  import { editorHref } from '#lib/editor/navigation.ts';
  import { clusterItems, tileDetails, topMarks, type TileFacts } from '#lib/browse/tileContent.ts';
  import { tileLayout } from '#lib/browse/tileLayout.ts';
  import type { TileInfo, TileOverlay } from '#lib/browse/tileOverlay.ts';
  import { Icon } from '@immich/ui';
  import { mdiCheck, mdiPencilOutline } from '@mdi/js';
  import { held } from '#lib/input/modifiers.svelte.ts';

  let {
    asset,
    info = 'icons',
    overlay,
    facts,
    width,
    height,
    selected = false,
    rangePreview = false,
    selectionActive = false,
    selectMode = false,
    onToggle,
    onRange,
    onPreview,
    onPreviewEnd,
    onActivate,
    onLoupe,
    onDeleteCopy,
    onRate
  }: {
    asset: AssetSummary;
    info?: TileInfo;
    overlay: TileOverlay;
    facts: TileFacts;
    width: number;
    height: number;
    selected?: boolean;
    rangePreview?: boolean;
    selectionActive?: boolean;
    selectMode?: boolean;
    onToggle?: () => void;
    onRange?: () => void;
    onPreview?: () => void;
    onPreviewEnd?: () => void;
    onActivate?: () => void;
    onLoupe?: () => void;
    onDeleteCopy?: () => void;
    onRate?: (rating: number | null) => void;
  } = $props();

  let pendingDelete = $state(false);

  const rating = $derived(asset.exifInfo?.rating ?? 0);
  const owned = $derived(ownsAsset(asset));
  const rejected = $derived(isRejected(asset));
  const label = $derived(labelOf(asset));
  const src = $derived(assetThumbUrl(asset));
  const placeholder = $derived(assetPlaceholder(asset));
  const marked = $derived(selected || rangePreview);
  const shape = $derived(tileLayout(info, overlay, { width, height }));
  const marks = $derived(topMarks(overlay, asset, facts));
  const cluster = $derived(clusterItems(overlay, asset, facts));
  const details = $derived(tileDetails(overlay, asset, facts, shape.lines));
  const hoverButton = {
    size: 'small',
    shape: 'round',
    variant: 'ghost',
    color: 'secondary',
    class: 'text-white drop-shadow-md bg-black/40 hover:bg-black/70'
  } as const;
  const hoverOnly = 'opacity-0 group-hover:opacity-100 group-has-[:focus-visible]:opacity-100';
  const checkClass = $derived(
    marked || selectMode
      ? 'opacity-100'
      : selectionActive
        ? 'opacity-0 group-hover:opacity-100 touch:opacity-100'
        : 'opacity-0 group-hover:opacity-100 touch:hidden'
  );
  const href = $derived(editorHref(asset.id, currentPath()));

  function onClick(e: MouseEvent): void {
    if (held('extend', e)) {
      e.preventDefault();
      onRange?.();
      return;
    }
    if (held('toggle', e)) {
      e.preventDefault();
      onToggle?.();
      return;
    }
    if (selectionActive) {
      e.preventDefault();
      onToggle?.();
      return;
    }
    e.preventDefault();
    onActivate?.();
    onLoupe?.();
  }

  function onCheckbox(e: MouseEvent): void {
    e.preventDefault();
    e.stopPropagation();
    if (held('extend', e)) {
      onRange?.();
      return;
    }
    onToggle?.();
  }

  const drag = tileDrag();
  const draggable = $derived(drag !== null && layout.mode !== 'touch');
</script>

<div
  role="group"
  draggable={draggable || undefined}
  ondragstart={draggable ? (event) => drag?.start(asset, event) : undefined}
  ondragend={draggable ? () => drag?.end() : undefined}
  onpointerenter={onPreview}
  onpointerleave={onPreviewEnd}
  class="group no-callout relative isolate block h-full w-full overflow-hidden transition-[background-color,border-radius] duration-base"
  class:bg-light-100={!selected}
  class:bg-light-400={selected}
  class:rounded-md={selected}
  data-selected={selected || undefined}
  data-range-preview={rangePreview || undefined}
  data-name={asset.originalFileName}
  data-asset-id={asset.id}
>
  <a
    {href}
    onclick={onClick}
    aria-label={asset.originalFileName}
    class="block h-full w-full outline-none transition-[padding] duration-base focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary"
    class:p-2={selected}
    draggable="false"
  >
    <div class="h-full w-full overflow-hidden" class:rounded-sm={selected}>
      <img
        {src}
        alt=""
        loading="lazy"
        draggable="false"
        class="h-full w-full bg-cover bg-center object-cover transition-[filter,transform] duration-base group-hover:scale-[1.01]"
        style:transform={selected ? 'none' : undefined}
        style:background-image={placeholder}
        class:opacity-40={rejected}
        class:grayscale={rejected}
      />
    </div>
  </a>
  {#if rangePreview && !selected}
    <div class="pointer-events-none absolute inset-0 bg-primary/25" aria-hidden="true"></div>
  {/if}
  <button
    type="button"
    onclick={onCheckbox}
    aria-label={selected ? 'Deselect' : 'Select'}
    aria-pressed={selected}
    class="absolute top-2 left-2 flex size-6 items-center justify-center rounded-full border-2 border-white bg-black/35 text-white drop-shadow-md transition-[opacity,background-color,border-color,transform] duration-fast hover:scale-105 focus-visible:opacity-100 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary {checkClass}"
    class:border-primary={marked}
    class:bg-primary={marked}
  >
    {#if marked}
      <Icon icon={mdiCheck} size="16px" aria-hidden="true" />
    {/if}
  </button>
  {#if marks.length > 0}
    <TileMarks {marks} {label} low={shape.low} class={info === 'none' ? hoverOnly : ''} />
  {/if}
  {#if !selectMode}
    <div
      class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 flex items-center gap-1.5 transition-opacity opacity-0 group-hover:opacity-100 touch:hidden"
    >
      <IconLink
        {...hoverButton}
        icon={mdiPencilOutline}
        {href}
        tooltip={hint('Edit', 'openEditor')}
        aria-label="Edit"
        onclick={() => onActivate?.()}
      />
      {#if isCopy(asset.id) && onDeleteCopy}
        <DeleteConfirmation
          bind:pending={pendingDelete}
          label="Delete copy"
          title="Delete this virtual copy"
          confirmLabel="Confirm delete copy"
          size="small"
          round
          deleteClass="drop-shadow-md bg-black/40 text-white hover:bg-black/70 hover:text-red-400"
          confirmClass="drop-shadow-md bg-red-500/80 text-white hover:bg-red-500"
          onconfirm={onDeleteCopy}
        />
      {/if}
    </div>
  {/if}
  <TileBand {info} {shape} {details} {cluster} {rating} {owned} {onRate} />
</div>
