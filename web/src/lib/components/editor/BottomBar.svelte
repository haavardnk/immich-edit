<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import {
    mdiHeart,
    mdiHeartOutline,
    mdiMagnifyMinusOutline,
    mdiMagnifyPlusOutline,
    mdiStar,
    mdiStarOutline
  } from '@mdi/js';

  const exif = $derived(editor.asset?.exifInfo ?? null);
  const dims = $derived(
    exif?.exifImageWidth && exif?.exifImageHeight
      ? `${exif.exifImageWidth} × ${exif.exifImageHeight}`
      : ''
  );
  const rating = $derived(exif?.rating ?? 0);
  const isFav = $derived(editor.asset?.isFavorite ?? false);
  const hasAsset = $derived(editor.asset != null);

  function clickStar(n: number): void {
    if (rating === n) {
      void editor.setRating(null);
    } else {
      void editor.setRating(n);
    }
  }
</script>

<div class="grid grid-cols-3 items-center px-3 py-1 bg-immich-dark-bg/80 backdrop-blur-sm border-t border-white/5">
  <div class="flex items-center gap-2 text-[11px] font-mono text-immich-dark-fg/50 justify-self-start">
    {dims}
  </div>

  <div class="flex items-center gap-1 justify-self-center">
    {#if hasAsset}
      {#each [1, 2, 3, 4, 5] as n (n)}
        <button
          class="btn btn-ghost btn-xs btn-square"
          title={`${n} star${n > 1 ? 's' : ''}`}
          onclick={() => clickStar(n)}
        >
          <Icon path={n <= rating ? mdiStar : mdiStarOutline} size={16} />
        </button>
      {/each}
      <button
        class="btn btn-ghost btn-xs btn-square ml-2"
        title={isFav ? 'Unfavorite' : 'Favorite'}
        onclick={() => editor.toggleFavorite()}
      >
        <Icon path={isFav ? mdiHeart : mdiHeartOutline} size={16} />
      </button>
    {/if}
  </div>

  <div class="flex items-center gap-1.5 justify-self-end">
    <button
      class="btn btn-ghost btn-xs btn-square"
      title="Zoom Out"
      onclick={ui.zoomOut}
    >
      <Icon path={mdiMagnifyMinusOutline} size={16} />
    </button>
    <input
      type="range"
      min="25"
      max="400"
      step="5"
      value={ui.zoom}
      oninput={(e: Event) => ui.setZoom(Number((e.target as HTMLInputElement).value))}
      class="range range-xs w-24"
    />
    <button
      class="btn btn-ghost btn-xs btn-square"
      title="Zoom In"
      onclick={ui.zoomIn}
    >
      <Icon path={mdiMagnifyPlusOutline} size={16} />
    </button>
    <button
      class="btn btn-ghost btn-xs min-w-14 font-mono text-xs"
      title="Fit to Screen"
      onclick={ui.zoomFit}
    >
      {ui.zoom}%
    </button>
  </div>
</div>
