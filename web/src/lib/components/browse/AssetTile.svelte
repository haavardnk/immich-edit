<script lang="ts">
  import { page } from '$app/state';
  import DeleteConfirmation from '#lib/components/DeleteConfirmation.svelte';
  import IconLink from '#lib/components/IconLink.svelte';
  import StarRating from '#lib/components/StarRating.svelte';
  import RatingBadge from '#lib/components/browse/RatingBadge.svelte';
  import DisabledReason from '#lib/components/DisabledReason.svelte';
  import { NOT_OWNER, ownsAsset } from '#lib/browse/ownership.ts';
  import type { AssetSummary } from '#lib/types/album.ts';
  import { hint } from '#lib/shortcuts/labels.ts';
  import { assetPlaceholder, assetThumbUrl } from '#lib/api/assets.ts';
  import { copyIndex, isCopy } from '#lib/browse/assetKey.ts';
  import { isRejected } from '#lib/browse/reject.ts';
  import { LABEL_NAMES, LABEL_TEXT, labelOf } from '#lib/stores/labels.ts';
  import { editorHref } from '#lib/editor/navigation.ts';
  import { captureTime, fmtDay } from '#lib/utils/exif.ts';
  import type { TileInfo } from '#lib/stores/browseView.svelte.ts';
  import { editedThumbs } from '#lib/stores/editedThumbs.svelte.ts';
  import { Icon, IconButton } from '@immich/ui';
  import {
    mdiHeart,
    mdiCircle,
    mdiCheck,
    mdiEyeOutline,
    mdiCloseCircle,
    mdiContentDuplicate,
    mdiPencilOutline,
    mdiTuneVariant
  } from '@mdi/js';
  import { held } from '#lib/input/modifiers.svelte.ts';

  let {
    asset,
    info = 'badges',
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
    onOpen,
    onDeleteCopy,
    onRate
  }: {
    asset: AssetSummary;
    info?: TileInfo;
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
    onOpen?: () => void;
    onDeleteCopy?: () => void;
    onRate?: (rating: number | null) => void;
  } = $props();

  let pendingDelete = $state(false);

  const rating = $derived(asset.exifInfo?.rating ?? 0);
  const owned = $derived(ownsAsset(asset));
  const rejected = $derived(isRejected(asset));
  const label = $derived(labelOf(asset));
  const edited = $derived(info !== 'none' && editedThumbs.has(asset.id));
  const src = $derived(assetThumbUrl(asset));
  const placeholder = $derived(assetPlaceholder(asset));
  const marked = $derived(selected || rangePreview);
  const copyBadge = $derived(
    isCopy(asset.id) ? (asset.copyLabel ?? `Copy ${copyIndex(asset.id)}`) : null
  );
  const rowPinned = $derived(
    info === 'full' || (info === 'badges' && (copyBadge !== null || rating > 0))
  );
  const hoverButton = {
    size: 'small',
    shape: 'round',
    variant: 'ghost',
    color: 'secondary',
    class: 'text-white drop-shadow-md bg-black/40 hover:bg-black/70'
  } as const;
  const hoverOnly = 'opacity-0 group-hover:opacity-100 group-has-[:focus-visible]:opacity-100';
  const markClass = $derived(info === 'none' ? hoverOnly : '');
  const nameClass = $derived(info === 'full' ? '' : hoverOnly);
  const checkClass = $derived(
    marked || selectMode
      ? 'opacity-100'
      : selectionActive
        ? 'opacity-0 group-hover:opacity-100 touch:opacity-100'
        : 'opacity-0 group-hover:opacity-100 touch:hidden'
  );
  const taken = $derived(info === 'full' ? fmtDay(captureTime(asset)) : null);
  const href = $derived(editorHref(asset.id, `${page.url.pathname}${page.url.search}`));

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
    onActivate?.();
    if (!onOpen) return;
    e.preventDefault();
    onOpen();
  }

  function onLoupeClick(e: MouseEvent): void {
    e.preventDefault();
    e.stopPropagation();
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
</script>

<div
  role="group"
  onpointerenter={onPreview}
  onpointerleave={onPreviewEnd}
  class="group no-callout relative isolate block h-full w-full overflow-hidden transition-[background-color,border-radius] duration-200"
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
    class="block h-full w-full outline-none transition-[padding] duration-200 focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary"
    class:p-2={selected}
    draggable="false"
  >
    <div class="h-full w-full overflow-hidden" class:rounded-sm={selected}>
      <img
        {src}
        alt=""
        loading="lazy"
        draggable="false"
        class="h-full w-full bg-cover bg-center object-cover transition-[filter,transform] duration-200 group-hover:scale-[1.01]"
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
    class="absolute top-2 left-2 flex size-6 items-center justify-center rounded-full border-2 border-white bg-black/35 text-white drop-shadow-md transition-[opacity,background-color,border-color,transform] duration-150 hover:scale-105 focus-visible:opacity-100 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary {checkClass}"
    class:border-primary={marked}
    class:bg-primary={marked}
  >
    {#if marked}
      <Icon icon={mdiCheck} size="16px" aria-hidden="true" />
    {/if}
  </button>
  {#if edited || asset.isFavorite || rejected || label}
    <div
      class="pointer-events-none absolute top-1 right-1 flex flex-col items-center gap-1 text-white drop-shadow-md"
    >
      {#if edited}
        <div
          role="img"
          aria-label="Edited"
          class="grid size-5 place-items-center rounded-full bg-black/55"
        >
          <Icon icon={mdiTuneVariant} size="12px" aria-hidden="true" />
        </div>
      {/if}
      {#if asset.isFavorite}
        <div role="img" aria-label="Favorite" class="transition-opacity {markClass}">
          <Icon icon={mdiHeart} size="16px" aria-hidden="true" />
        </div>
      {/if}
      {#if rejected}
        <div role="img" aria-label="Rejected" class="transition-opacity {markClass}">
          <Icon icon={mdiCloseCircle} size="16px" aria-hidden="true" />
        </div>
      {/if}
      {#if label}
        <div
          role="img"
          aria-label="{LABEL_NAMES[label]} label"
          class="transition-opacity {LABEL_TEXT[label]} {markClass}"
        >
          <Icon icon={mdiCircle} size="14px" aria-hidden="true" />
        </div>
      {/if}
    </div>
  {/if}
  {#if !selectMode}
    <div
      class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 flex items-center gap-1.5 transition-opacity opacity-0 group-hover:opacity-100 touch:hidden"
    >
      {#if onOpen}
        <IconLink
          {...hoverButton}
          icon={mdiPencilOutline}
          {href}
          tooltip={hint('Edit', 'openEditor')}
          aria-label="Edit"
          onclick={() => onActivate?.()}
        />
      {:else}
        <IconButton
          {...hoverButton}
          type="button"
          icon={mdiEyeOutline}
          title={hint('Quick review', 'openLoupe')}
          aria-label="Quick review"
          onclick={onLoupeClick}
        />
      {/if}
      {#if copyBadge && onDeleteCopy}
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
  <div
    data-testid="tile-info"
    class="pointer-events-none absolute inset-x-0 bottom-0 flex min-h-8 flex-col justify-end gap-0.5 bg-linear-to-t from-black/90 via-black/55 to-transparent px-2 pb-1.5 pt-4 text-white transition-opacity {rowPinned
      ? 'opacity-100'
      : 'opacity-0 group-hover:opacity-100 group-has-[:focus-visible]:opacity-100'}"
  >
    {#if onRate}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <span
        class="pointer-events-auto -ml-1 self-start drop-shadow-md {info === 'full'
          ? 'flex'
          : 'hidden group-hover:flex group-has-[:focus-visible]:flex'}"
        onkeydown={(e) => e.stopPropagation()}
        onmousedown={(e) => e.preventDefault()}
      >
        <DisabledReason locked={!owned} reason={NOT_OWNER}>
          <StarRating {rating} size={12} disabled={!owned} onchange={onRate} />
        </DisabledReason>
      </span>
    {/if}
    <div class="flex items-end gap-2">
      <span
        data-testid="tile-name"
        class="flex min-w-0 flex-1 flex-col drop-shadow-md transition-opacity {nameClass}"
      >
        <span class="truncate text-[10px] font-medium">{asset.originalFileName}</span>
        {#if taken}
          <span class="truncate text-[9px] text-white/70">{taken}</span>
        {/if}
      </span>
      {#if copyBadge}
        <span class="flex max-w-[45%] shrink-0 items-center gap-1 text-[9px] text-white/75">
          <Icon icon={mdiContentDuplicate} size="11px" aria-hidden="true" />
          <span class="truncate">{copyBadge}</span>
        </span>
      {/if}
      {#if rating > 0 && (!onRate || info !== 'full')}
        <RatingBadge
          {rating}
          class={onRate ? 'group-hover:hidden group-has-[:focus-visible]:hidden' : ''}
        />
      {/if}
    </div>
  </div>
</div>
