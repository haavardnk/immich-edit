<script lang="ts">
  import { page } from '$app/state';
  import { browsing } from '$lib/stores/browsing.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { assetThumbUrl } from '$lib/api/assets';
  import Icon from '$lib/components/Icon.svelte';
  import { isRejected } from '$lib/reject';
  import { mdiChevronDown, mdiChevronUp, mdiCloseCircle, mdiHeart, mdiStar } from '@mdi/js';

  let {
    currentId: currentIdProp = null,
    onSelect,
    size = 64,
    showBadges = false,
    highlightIds
  }: {
    currentId?: string | null;
    onSelect?: (id: string) => void;
    size?: number;
    showBadges?: boolean;
    highlightIds?: string[];
  } = $props();

  const GAP = 4;
  const PAD = 8;
  const OVERSCAN = 3;
  const STRIDE = $derived(size + GAP);

  const currentId = $derived(onSelect ? currentIdProp : (page.params.id ?? null));

  const assets = $derived(browsing.assets);
  const currentIndex = $derived(assets.findIndex((a) => a.id === currentId));

  let scrollContainer: HTMLDivElement | undefined = $state();
  let containerWidth = $state(0);
  let scrollLeft = $state(0);

  const totalWidth = $derived(PAD * 2 + Math.max(0, assets.length) * STRIDE - GAP);

  const view = $derived.by(() => {
    const startIdx = Math.max(0, Math.floor((scrollLeft - PAD) / STRIDE) - OVERSCAN);
    const count = Math.ceil(containerWidth / STRIDE) + OVERSCAN * 2;
    return {
      startIdx,
      endIdx: Math.min(assets.length, startIdx + count),
      offsetX: PAD + startIdx * STRIDE
    };
  });

  const visibleAssets = $derived(assets.slice(view.startIdx, view.endIdx));

  function measure(): void {
    if (!scrollContainer) return;
    containerWidth = scrollContainer.clientWidth;
    scrollLeft = scrollContainer.scrollLeft;
  }

  $effect(() => {
    if (!scrollContainer) return;
    measure();
    const ro = new ResizeObserver(() => measure());
    ro.observe(scrollContainer);
    return () => ro.disconnect();
  });

  $effect(() => {
    if (!scrollContainer || currentIndex < 0) return;
    const left = PAD + currentIndex * STRIDE;
    if (left >= scrollLeft && left + size <= scrollLeft + containerWidth) return;
    const target = left + size / 2 - containerWidth / 2;
    const max = Math.max(0, totalWidth - containerWidth);
    scrollContainer.scrollTo({ left: Math.min(Math.max(0, target), max), behavior: 'smooth' });
  });
</script>

{#if assets.length > 0}
  <div class="border-t border-white/5 bg-immich-dark-gray flex-none">
    {#if ui.filmstripCollapsed}
      <button
        class="w-full flex items-center justify-center h-5 hover:bg-white/5 transition-colors"
        onclick={ui.toggleFilmstrip}
        aria-label="expand filmstrip"
        title="Filmstrip"
      >
        <Icon path={mdiChevronUp} size={14} class="opacity-40" />
      </button>
    {:else}
      <div class="relative">
        <button
          class="absolute right-1 top-1 z-10 p-0.5 rounded bg-black/40 hover:bg-white/15 transition-colors"
          onclick={ui.toggleFilmstrip}
          aria-label="collapse filmstrip"
          title="Collapse"
        >
          <Icon path={mdiChevronDown} size={14} class="opacity-70" />
        </button>
        <div
          class="py-2 overflow-x-auto scrollbar-hidden"
          bind:this={scrollContainer}
          onscroll={measure}
        >
          <div class="relative" style:width="{totalWidth}px" style:height="{size}px">
            <div class="absolute top-0 flex gap-1" style:left="{view.offsetX}px">
              {#each visibleAssets as asset (asset.id)}
                {@const isCurrent = asset.id === currentId}
                {@const paneNumber = (highlightIds?.indexOf(asset.id) ?? -1) + 1}
                {@const isMember = !isCurrent && paneNumber > 0}
                {@const rating = asset.exifInfo?.rating ?? 0}
                {@const rejected = isRejected(asset)}
                {#if onSelect}
                  <button
                    type="button"
                    onclick={() => onSelect(asset.id)}
                    class="group relative flex-none rounded-lg overflow-hidden transition-all {isCurrent
                      ? 'ring-2 ring-immich-dark-primary'
                      : isMember
                        ? 'ring-2 ring-immich-dark-primary/50'
                        : ''}"
                    style:width="{size}px"
                    style:height="{size}px"
                    title={asset.originalFileName}
                    aria-label={asset.originalFileName}
                  >
                    <img
                      src={assetThumbUrl(asset.id)}
                      alt=""
                      loading="lazy"
                      class="w-full h-full object-cover transition-opacity {isCurrent || isMember
                        ? 'opacity-100'
                        : 'opacity-50 group-hover:opacity-80'}"
                      class:grayscale={rejected}
                    />
                    {#if paneNumber > 0}
                      <span
                        class="absolute left-1 top-1 min-w-4 px-1 rounded text-[10px] leading-4 font-medium text-center pointer-events-none {isCurrent
                          ? 'bg-immich-dark-primary text-black'
                          : 'bg-black/60 text-white/70'}"
                      >
                        {paneNumber}
                      </span>
                    {/if}
                    {#if showBadges && asset.isFavorite}
                      <div class="absolute top-1 right-1 text-white drop-shadow-md pointer-events-none">
                        <Icon path={mdiHeart} size={13} />
                      </div>
                    {/if}
                    {#if showBadges && rejected}
                      <div
                        class="absolute right-1 text-white drop-shadow-md pointer-events-none"
                        class:top-1={!asset.isFavorite}
                        class:top-6={asset.isFavorite}
                      >
                        <Icon path={mdiCloseCircle} size={13} />
                      </div>
                    {/if}
                    {#if showBadges && rating > 0}
                      <div
                        class="absolute inset-x-0 bottom-0 flex items-end px-1 pb-1 pt-3 bg-linear-to-t from-black/75 to-transparent text-white drop-shadow-md pointer-events-none"
                      >
                        <div class="flex items-center gap-0.5">
                          {#each [1, 2, 3, 4, 5] as n (n)}
                            <Icon path={mdiStar} size={9} class={n <= rating ? 'opacity-100' : 'opacity-30'} />
                          {/each}
                        </div>
                      </div>
                    {/if}
                  </button>
                {:else}
                  <a
                    href={`/assets/${asset.id}`}
                    class="group relative flex-none rounded-lg overflow-hidden transition-all {isCurrent
                      ? 'ring-2 ring-immich-dark-primary'
                      : ''}"
                    style:width="{size}px"
                    style:height="{size}px"
                    title={asset.originalFileName}
                  >
                    <img
                      src={assetThumbUrl(asset.id)}
                      alt=""
                      loading="lazy"
                      class="w-full h-full object-cover transition-opacity {isCurrent
                        ? 'opacity-100'
                        : 'opacity-50 group-hover:opacity-80'}"
                      class:grayscale={rejected}
                    />
                    {#if showBadges && asset.isFavorite}
                      <div class="absolute top-1 right-1 text-white drop-shadow-md pointer-events-none">
                        <Icon path={mdiHeart} size={13} />
                      </div>
                    {/if}
                    {#if showBadges && rejected}
                      <div
                        class="absolute right-1 text-white drop-shadow-md pointer-events-none"
                        class:top-1={!asset.isFavorite}
                        class:top-6={asset.isFavorite}
                      >
                        <Icon path={mdiCloseCircle} size={13} />
                      </div>
                    {/if}
                    {#if showBadges && rating > 0}
                      <div
                        class="absolute inset-x-0 bottom-0 flex items-end px-1 pb-1 pt-3 bg-linear-to-t from-black/75 to-transparent text-white drop-shadow-md pointer-events-none"
                      >
                        <div class="flex items-center gap-0.5">
                          {#each [1, 2, 3, 4, 5] as n (n)}
                            <Icon path={mdiStar} size={9} class={n <= rating ? 'opacity-100' : 'opacity-30'} />
                          {/each}
                        </div>
                      </div>
                    {/if}
                  </a>
                {/if}
              {/each}
            </div>
          </div>
        </div>
      </div>
    {/if}
  </div>
{/if}
