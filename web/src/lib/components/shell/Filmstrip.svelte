<script lang="ts">
  import { page } from '$app/state';
  import { browsing } from '$lib/stores/browsing.svelte';
  import { MAX_FILMSTRIP_HEIGHT, MIN_FILMSTRIP_HEIGHT, ui } from '$lib/stores/ui.svelte';
  import { assetThumbUrl } from '$lib/api/assets';
  import { createFilmstripLayout, visibleFilmstripRange } from '$lib/filmstripLayout';
  import { isRejected } from '$lib/reject';
  import { editorHref } from '$lib/editorNavigation';
  import { Icon } from '@immich/ui';
  import { mdiCloseCircle, mdiHeart, mdiStar } from '@mdi/js';

  let {
    currentId: currentIdProp = null,
    onSelect,
    size = 64,
    resizable = false,
    showBadges = false,
    highlightIds,
    collapsed = false
  }: {
    currentId?: string | null;
    onSelect?: (id: string, additive: boolean) => void;
    size?: number;
    resizable?: boolean;
    showBadges?: boolean;
    highlightIds?: string[];
    collapsed?: boolean;
  } = $props();

  const GAP = 4;
  const PAD = 8;
  const OVERSCAN = 3;
  const currentId = $derived(onSelect ? currentIdProp : (page.params.id ?? null));
  const thumbnailHeight = $derived(resizable ? ui.filmstripHeight : size);

  const assets = $derived(browsing.assets);
  const currentIndex = $derived(assets.findIndex((a) => a.id === currentId));
  const layout = $derived(createFilmstripLayout(assets, thumbnailHeight, GAP, PAD));

  let scrollContainer: HTMLDivElement | undefined = $state();
  let containerWidth = $state(0);
  let scrollLeft = $state(0);
  let resizing = $state(false);
  let resizeStartY = 0;
  let resizeStartHeight = 0;

  const view = $derived.by(() => {
    const range = visibleFilmstripRange(
      layout.boxes,
      scrollLeft,
      scrollLeft + containerWidth,
      OVERSCAN
    );
    return {
      startIdx: range.startIndex,
      endIdx: range.endIndex,
      offsetX: layout.boxes[range.startIndex]?.left ?? PAD
    };
  });

  const visibleAssets = $derived(assets.slice(view.startIdx, view.endIdx));

  function measure(): void {
    if (!scrollContainer) return;
    containerWidth = scrollContainer.clientWidth;
    scrollLeft = scrollContainer.scrollLeft;
  }

  function startResize(event: PointerEvent): void {
    if (!resizable) return;
    resizing = true;
    resizeStartY = event.clientY;
    resizeStartHeight = ui.filmstripHeight;
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  }

  function resize(event: PointerEvent): void {
    if (!resizing) return;
    ui.setFilmstripHeight(resizeStartHeight + resizeStartY - event.clientY);
  }

  function stopResize(event: PointerEvent): void {
    if (!resizing) return;
    resizing = false;
    ui.persistEditorUi();
    (event.currentTarget as HTMLElement).releasePointerCapture(event.pointerId);
  }

  function resizeWithKeyboard(event: KeyboardEvent): void {
    if (!resizable) return;
    const amount = event.shiftKey ? 16 : 8;
    if (event.key === 'ArrowUp') ui.setFilmstripHeight(ui.filmstripHeight + amount);
    else if (event.key === 'ArrowDown') ui.setFilmstripHeight(ui.filmstripHeight - amount);
    else if (event.key === 'Home') ui.setFilmstripHeight(MIN_FILMSTRIP_HEIGHT);
    else if (event.key === 'End') ui.setFilmstripHeight(MAX_FILMSTRIP_HEIGHT);
    else return;
    ui.persistEditorUi();
    event.preventDefault();
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
    const box = layout.boxes[currentIndex];
    if (box.left >= scrollLeft && box.left + box.width <= scrollLeft + containerWidth) return;
    const target = box.left + box.width / 2 - containerWidth / 2;
    const max = Math.max(0, layout.width - containerWidth);
    scrollContainer.scrollTo({ left: Math.min(Math.max(0, target), max), behavior: 'smooth' });
  });
</script>

{#if assets.length > 0}
  <div class="relative flex-none border-t border-white/12 bg-editor-chrome shadow-filmstrip">
    {#if resizable && !collapsed}
      <div
        role="slider"
        tabindex="0"
        aria-label="Resize filmstrip"
        aria-orientation="vertical"
        aria-valuemin={MIN_FILMSTRIP_HEIGHT}
        aria-valuemax={MAX_FILMSTRIP_HEIGHT}
        aria-valuenow={ui.filmstripHeight}
        class="absolute inset-x-0 top-0 z-20 h-2 -translate-y-1/2 cursor-row-resize outline-none focus-visible:bg-primary/30"
        onpointerdown={startResize}
        onpointermove={resize}
        onpointerup={stopResize}
        onpointercancel={stopResize}
        onkeydown={resizeWithKeyboard}
      ></div>
    {/if}
    {#if !collapsed}
      <div class="relative">
        <div
          class="overflow-x-auto py-1.5 scrollbar-hidden"
          bind:this={scrollContainer}
          onscroll={measure}
        >
          <div class="relative" style:width="{layout.width}px" style:height="{thumbnailHeight}px">
            <div class="absolute top-0 flex gap-1" style:left="{view.offsetX}px">
              {#each visibleAssets as asset, visibleIndex (asset.id)}
                {@const box = layout.boxes[view.startIdx + visibleIndex]}
                {@const isCurrent = asset.id === currentId}
                {@const paneNumber = (highlightIds?.indexOf(asset.id) ?? -1) + 1}
                {@const isMember = !isCurrent && paneNumber > 0}
                {@const rating = asset.exifInfo?.rating ?? 0}
                {@const rejected = isRejected(asset)}
                {#if onSelect}
                  <button
                    type="button"
                    onclick={(e) => onSelect(asset.id, e.metaKey || e.ctrlKey || e.shiftKey)}
                    aria-pressed={isCurrent || isMember}
                    class="group relative flex-none overflow-hidden rounded-sm border-2 bg-neutral-900 outline-none transition-[opacity,border-color,box-shadow] focus-visible:ring-2 focus-visible:ring-primary {isCurrent
                      ? 'border-primary ring-2 ring-primary/60'
                      : isMember
                        ? 'border-primary/75 ring-1 ring-primary/40'
                        : 'border-transparent hover:border-white/25'}"
                    style:width="{box.width}px"
                    style:height="{thumbnailHeight}px"
                    title={asset.originalFileName}
                    aria-label={asset.originalFileName}
                  >
                    <img
                      src={assetThumbUrl(asset.id)}
                      alt=""
                      loading="lazy"
                      class="w-full h-full object-cover transition-[opacity,filter] {isCurrent ||
                      isMember
                        ? 'opacity-100'
                        : 'opacity-55 saturate-75 group-hover:opacity-95 group-hover:saturate-100'}"
                      class:grayscale={rejected}
                    />
                    {#if paneNumber > 0}
                      <span
                        class="pointer-events-none absolute top-1 left-1 min-w-5 rounded border px-1 text-center text-[10px] leading-4 font-semibold shadow-sm {isCurrent
                          ? 'border-primary bg-primary text-neutral-950'
                          : 'border-primary/70 bg-neutral-950/90 text-primary'}"
                      >
                        {paneNumber}
                      </span>
                    {/if}
                    {#if showBadges && asset.isFavorite}
                      <div
                        class="absolute top-1 right-1 text-white drop-shadow-md pointer-events-none"
                      >
                        <Icon icon={mdiHeart} size="13px" aria-hidden="true" />
                      </div>
                    {/if}
                    {#if showBadges && rejected}
                      <div
                        class="absolute right-1 text-white drop-shadow-md pointer-events-none"
                        class:top-1={!asset.isFavorite}
                        class:top-6={asset.isFavorite}
                      >
                        <Icon icon={mdiCloseCircle} size="13px" aria-hidden="true" />
                      </div>
                    {/if}
                    {#if showBadges && rating > 0}
                      <div
                        class="absolute inset-x-0 bottom-0 flex items-end px-1 pb-1 pt-3 bg-linear-to-t from-black/75 to-transparent text-white drop-shadow-md pointer-events-none"
                      >
                        <div class="flex items-center gap-0.5">
                          {#each [1, 2, 3, 4, 5] as n (n)}
                            <Icon
                              icon={mdiStar}
                              size="9px"
                              class={n <= rating ? 'opacity-100' : 'opacity-30'}
                              aria-hidden="true"
                            />
                          {/each}
                        </div>
                      </div>
                    {/if}
                  </button>
                {:else}
                  <a
                    href={editorHref(asset.id, page.url.searchParams.get('from'))}
                    aria-label={asset.originalFileName}
                    aria-current={isCurrent ? 'page' : undefined}
                    class="group relative flex-none overflow-hidden rounded-sm border bg-neutral-900 outline-none transition-[opacity,border-color,box-shadow] focus-visible:ring-2 focus-visible:ring-primary {isCurrent
                      ? 'border-primary ring-1 ring-primary/40'
                      : 'border-hairline hover:border-white/25'}"
                    style:width="{box.width}px"
                    style:height="{thumbnailHeight}px"
                    title={asset.originalFileName}
                  >
                    <img
                      src={assetThumbUrl(asset.id)}
                      alt=""
                      loading="lazy"
                      class="w-full h-full object-cover transition-[opacity,filter] {isCurrent
                        ? 'opacity-100'
                        : 'opacity-65 saturate-75 group-hover:opacity-95 group-hover:saturate-100'}"
                      class:grayscale={rejected}
                    />
                    {#if showBadges && asset.isFavorite}
                      <div
                        class="absolute top-1 right-1 text-white drop-shadow-md pointer-events-none"
                      >
                        <Icon icon={mdiHeart} size="13px" aria-hidden="true" />
                      </div>
                    {/if}
                    {#if showBadges && rejected}
                      <div
                        class="absolute right-1 text-white drop-shadow-md pointer-events-none"
                        class:top-1={!asset.isFavorite}
                        class:top-6={asset.isFavorite}
                      >
                        <Icon icon={mdiCloseCircle} size="13px" aria-hidden="true" />
                      </div>
                    {/if}
                    {#if showBadges && rating > 0}
                      <div
                        class="absolute inset-x-0 bottom-0 flex items-end px-1 pb-1 pt-3 bg-linear-to-t from-black/75 to-transparent text-white drop-shadow-md pointer-events-none"
                      >
                        <div class="flex items-center gap-0.5">
                          {#each [1, 2, 3, 4, 5] as n (n)}
                            <Icon
                              icon={mdiStar}
                              size="9px"
                              class={n <= rating ? 'opacity-100' : 'opacity-30'}
                              aria-hidden="true"
                            />
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
