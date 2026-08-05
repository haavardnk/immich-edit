<script lang="ts">
  import type { AssetSummary } from '$lib/types/album';
  import { hint } from '$lib/keybinds';
  import { assetThumbUrl } from '$lib/api/assets';
  import Icon from '$lib/components/Icon.svelte';
  import { isRejected } from '$lib/reject';
  import {
    mdiHeart,
    mdiStar,
    mdiCheckCircle,
    mdiCheckboxBlankCircleOutline,
    mdiEyeOutline,
    mdiCloseCircle
  } from '@mdi/js';

  let {
    asset,
    active = false,
    selected = false,
    selectionActive = false,
    onToggle,
    onRange,
    onActivate,
    onLoupe
  }: {
    asset: AssetSummary;
    active?: boolean;
    selected?: boolean;
    selectionActive?: boolean;
    onToggle?: () => void;
    onRange?: () => void;
    onActivate?: () => void;
    onLoupe?: () => void;
  } = $props();

  const rating = $derived(asset.exifInfo?.rating ?? 0);
  const rejected = $derived(isRejected(asset));
  const src = $derived(assetThumbUrl(asset.id));

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
  class="block aspect-square overflow-hidden bg-white/5 rounded-lg group relative transition-all"
  class:ring-2={active || selected}
  class:ring-immich-dark-primary={active && !selected}
  class:ring-immich-primary={selected}
  title={asset.originalFileName}
>
  <a href={`/assets/${asset.id}`} onclick={onClick} class="block w-full h-full">
    <img
      src={src}
      alt=""
      loading="lazy"
      class="object-cover w-full h-full transition-transform group-hover:scale-105"
      class:opacity-70={selected}
      class:opacity-40={rejected}
      class:grayscale={rejected}
    />
  </a>
  <button
    type="button"
    onclick={onCheckbox}
    aria-label={selected ? 'Deselect' : 'Select'}
    class="absolute top-1 left-1 text-white drop-shadow-md transition-opacity {selected ||
    selectionActive
      ? 'opacity-100'
      : 'opacity-0 group-hover:opacity-100'}"
  >
    <Icon path={selected ? mdiCheckCircle : mdiCheckboxBlankCircleOutline} size={20} />
  </button>
  {#if asset.isFavorite}
    <div class="absolute top-1 right-1 text-white drop-shadow-md pointer-events-none">
      <Icon path={mdiHeart} size={16} />
    </div>
  {/if}
  {#if rejected}
    <div
      class="absolute right-1 text-white drop-shadow-md pointer-events-none"
      class:top-1={!asset.isFavorite}
      class:top-7={asset.isFavorite}
    >
      <Icon path={mdiCloseCircle} size={16} />
    </div>
  {/if}
  <button
    type="button"
    onclick={onLoupeClick}
    aria-label="Quick review"
    title={hint('Quick review', 'openLoupe')}
    class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 text-white drop-shadow-md rounded-full p-1.5 bg-black/40 hover:bg-black/70 transition-opacity opacity-0 group-hover:opacity-100"
  >
    <Icon path={mdiEyeOutline} size={22} />
  </button>
  {#if rating > 0}
    <div
      class="absolute bottom-1 left-1 flex items-center gap-0.5 text-white drop-shadow-md pointer-events-none"
    >
      {#each [1, 2, 3, 4, 5] as n (n)}
        <Icon path={mdiStar} size={12} class={n <= rating ? 'opacity-100' : 'opacity-30'} />
      {/each}
    </div>
  {/if}
  <div
    class="absolute inset-x-0 bottom-0 px-2 py-1 text-[10px] text-white truncate bg-linear-to-t from-black/70 to-transparent opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none"
  >
    {asset.originalFileName}
  </div>
</div>
