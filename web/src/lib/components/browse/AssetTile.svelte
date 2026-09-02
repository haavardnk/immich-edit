<script lang="ts">
  import { page } from '$app/state';
  import DeleteConfirmation from '$lib/components/DeleteConfirmation.svelte';
  import type { AssetSummary } from '$lib/types/album';
  import { hint } from '$lib/keybinds';
  import { assetThumbUrl } from '$lib/api/assets';
  import { copyIndex, isCopy } from '$lib/assetKey';
  import { isRejected } from '$lib/reject';
  import { editorHref } from '$lib/editorNavigation';
  import { Icon, IconButton } from '@immich/ui';
  import {
    mdiHeart,
    mdiStar,
    mdiCheck,
    mdiEyeOutline,
    mdiCloseCircle,
    mdiContentDuplicate
  } from '@mdi/js';

  let {
    asset,
    active = false,
    selected = false,
    rangePreview = false,
    selectionActive = false,
    onToggle,
    onRange,
    onPreview,
    onPreviewEnd,
    onActivate,
    onLoupe,
    onDeleteCopy
  }: {
    asset: AssetSummary;
    active?: boolean;
    selected?: boolean;
    rangePreview?: boolean;
    selectionActive?: boolean;
    onToggle?: () => void;
    onRange?: () => void;
    onPreview?: () => void;
    onPreviewEnd?: () => void;
    onActivate?: () => void;
    onLoupe?: () => void;
    onDeleteCopy?: () => void;
  } = $props();

  let pendingDelete = $state(false);

  const rating = $derived(asset.exifInfo?.rating ?? 0);
  const rejected = $derived(isRejected(asset));
  const src = $derived(assetThumbUrl(asset.id));
  const marked = $derived(selected || rangePreview);
  const copyBadge = $derived(
    isCopy(asset.id) ? (asset.copyLabel ?? `Copy ${copyIndex(asset.id)}`) : null
  );

  function onClick(e: MouseEvent): void {
    if (e.shiftKey) {
      e.preventDefault();
      onRange?.();
      return;
    }
    if (e.metaKey || e.ctrlKey) {
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
    if (e.shiftKey) {
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
  class="group relative isolate block h-full w-full overflow-hidden transition-[background-color,border-radius] duration-200"
  class:bg-neutral-900={!selected}
  class:bg-neutral-600={selected}
  class:rounded-md={selected}
  class:ring-2={active}
  class:ring-inset={active}
  class:ring-primary={active}
  data-selected={selected || undefined}
  data-range-preview={rangePreview || undefined}
  title={asset.originalFileName}
>
  <a
    href={editorHref(asset.id, `${page.url.pathname}${page.url.search}`)}
    onclick={onClick}
    aria-label={asset.originalFileName}
    class="block h-full w-full outline-none transition-[padding] duration-200 focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary"
    class:p-2={selected}
  >
    <div class="h-full w-full overflow-hidden" class:rounded-sm={selected}>
      <img
        {src}
        alt=""
        loading="lazy"
        class="h-full w-full object-cover transition-[filter,transform] duration-200 group-hover:scale-[1.01]"
        style:transform={selected ? 'none' : undefined}
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
    class="absolute top-2 left-2 flex size-6 items-center justify-center rounded-full border-2 border-white bg-black/35 text-white drop-shadow-md transition-[opacity,background-color,border-color,transform] duration-150 hover:scale-105 focus-visible:opacity-100 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary {marked
      ? 'opacity-100'
      : 'opacity-0 group-hover:opacity-100'}"
    class:border-primary={marked}
    class:bg-primary={marked}
  >
    {#if marked}
      <Icon icon={mdiCheck} size="16px" aria-hidden="true" />
    {/if}
  </button>
  {#if asset.isFavorite}
    <div
      role="img"
      aria-label="Favorite"
      class="absolute top-1 right-1 text-white drop-shadow-md pointer-events-none"
    >
      <Icon icon={mdiHeart} size="16px" aria-hidden="true" />
    </div>
  {/if}
  {#if rejected}
    <div
      role="img"
      aria-label="Rejected"
      class="absolute right-1 text-white drop-shadow-md pointer-events-none"
      class:top-1={!asset.isFavorite}
      class:top-7={asset.isFavorite}
    >
      <Icon icon={mdiCloseCircle} size="16px" aria-hidden="true" />
    </div>
  {/if}
  <div
    class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 flex items-center gap-1.5 transition-opacity opacity-0 group-hover:opacity-100"
  >
    <IconButton
      type="button"
      size="small"
      shape="round"
      variant="ghost"
      color="secondary"
      class="text-white drop-shadow-md bg-black/40 hover:bg-black/70"
      icon={mdiEyeOutline}
      title={hint('Quick review', 'openLoupe')}
      aria-label="Quick review"
      onclick={onLoupeClick}
    />
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
  <div
    class="pointer-events-none absolute inset-x-0 bottom-0 flex min-h-8 items-end gap-2 bg-linear-to-t from-black/90 via-black/55 to-transparent px-2 pb-1.5 pt-4 text-white transition-opacity {copyBadge ||
    rating > 0
      ? 'opacity-100'
      : 'opacity-0 group-hover:opacity-100 group-focus-within:opacity-100'}"
  >
    <span class="min-w-0 flex-1 truncate text-[10px] font-medium drop-shadow-md">
      {asset.originalFileName}
    </span>
    {#if copyBadge}
      <span class="flex max-w-[45%] shrink-0 items-center gap-1 text-[9px] text-white/75">
        <Icon icon={mdiContentDuplicate} size="11px" aria-hidden="true" />
        <span class="truncate">{copyBadge}</span>
      </span>
    {/if}
    {#if rating > 0}
      <span
        role="img"
        aria-label="{rating} star{rating === 1 ? '' : 's'}"
        class="flex shrink-0 items-center gap-0.5 text-[10px]"
      >
        <Icon icon={mdiStar} size="11px" aria-hidden="true" />
        {rating}
      </span>
    {/if}
  </div>
</div>
