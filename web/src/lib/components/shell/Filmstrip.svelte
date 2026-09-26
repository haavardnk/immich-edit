<script lang="ts">
  import { page } from '$app/state';
  import { untrack } from 'svelte';
  import { observeSize } from '$lib/actions/observeSize';
  import { browsing } from '$lib/stores/browsing.svelte';
  import { MAX_FILMSTRIP_HEIGHT, MIN_FILMSTRIP_HEIGHT, ui } from '$lib/stores/ui.svelte';
  import { assetThumbUrl } from '$lib/api/assets';
  import { createFilmstripLayout, visibleFilmstripRange } from '$lib/filmstripLayout';
  import { isRejected } from '$lib/reject';
  import { LABEL_NAMES, LABEL_TEXT, labelOf, type LabelColor } from '$lib/labels';
  import { editorHref } from '$lib/editorNavigation';
  import ResizeHandle from './ResizeHandle.svelte';
  import ContextMenu from '$lib/components/ContextMenu.svelte';
  import { mergeProps } from '$lib/utils/mergeProps';
  import type { Snippet } from 'svelte';
  import { Icon } from '@immich/ui';
  import { mdiCircle, mdiCloseCircle, mdiHeart, mdiStar } from '@mdi/js';

  let {
    currentId: currentIdProp = null,
    onSelect,
    size = 64,
    resizable = false,
    showBadges = false,
    highlightIds,
    collapsed = false,
    menu
  }: {
    currentId?: string | null;
    onSelect?: (id: string, additive: boolean) => void;
    size?: number;
    resizable?: boolean;
    showBadges?: boolean;
    highlightIds?: string[];
    collapsed?: boolean;
    menu?: Snippet<[string]>;
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

  $effect(() => {
    const last = containerWidth > 0 ? assets[view.endIdx - 1] : undefined;
    if (last) untrack(() => browsing.prefetchNear(last.id));
  });

  function measure(): void {
    if (!scrollContainer) return;
    containerWidth = scrollContainer.clientWidth;
    scrollLeft = scrollContainer.scrollLeft;
  }

  function onWheel(e: WheelEvent): void {
    if (!scrollContainer || e.deltaX !== 0 || e.deltaY === 0 || e.ctrlKey || e.metaKey) return;
    e.preventDefault();
    scrollContainer.scrollLeft += e.deltaY;
  }

  let revealed: { id: string; container: HTMLDivElement } | null = null;

  $effect(() => {
    const container = scrollContainer;
    const id = currentId;
    if (!container || !id || currentIndex < 0) return;
    const box = layout.boxes[currentIndex];
    if (!box) return;
    if (revealed?.id === id && revealed.container === container) return;
    revealed = { id, container };
    const viewLeft = container.scrollLeft;
    const viewWidth = container.clientWidth;
    if (box.left >= viewLeft && box.left + box.width <= viewLeft + viewWidth) return;
    const target = box.left + box.width / 2 - viewWidth / 2;
    const max = Math.max(0, layout.width - viewWidth);
    container.scrollTo({ left: Math.min(Math.max(0, target), max), behavior: 'smooth' });
  });
</script>

{#snippet badges(favorite: boolean, rejected: boolean, label: LabelColor | null)}
  {#if favorite || rejected || label}
    <div
      class="pointer-events-none absolute top-1 right-1 flex flex-col items-center gap-0.5 text-white drop-shadow-md"
    >
      {#if favorite}<Icon icon={mdiHeart} size="13px" aria-hidden="true" />{/if}
      {#if rejected}<Icon icon={mdiCloseCircle} size="13px" aria-hidden="true" />{/if}
      {#if label}
        <div role="img" aria-label="{LABEL_NAMES[label]} label" class={LABEL_TEXT[label]}>
          <Icon icon={mdiCircle} size="11px" aria-hidden="true" />
        </div>
      {/if}
    </div>
  {/if}
{/snippet}

{#if assets.length > 0}
  <div class="relative flex-none border-t border-white/12 bg-editor-chrome shadow-filmstrip">
    {#if resizable && !collapsed}
      <ResizeHandle
        label="Resize filmstrip"
        orientation="vertical"
        value={ui.filmstripHeight}
        min={MIN_FILMSTRIP_HEIGHT}
        max={MAX_FILMSTRIP_HEIGHT}
        step={8}
        shiftStep={16}
        class="absolute inset-x-0 top-0 z-20 h-2 -translate-y-1/2 cursor-row-resize outline-none focus-visible:bg-primary/30"
        onLive={ui.setFilmstripHeight}
        onCommit={ui.persistEditorUi}
      />
    {/if}
    {#if !collapsed}
      <div class="relative">
        <div
          class="overflow-x-auto py-1.5 scrollbar-hidden"
          data-testid="filmstrip-scroll"
          bind:this={scrollContainer}
          use:observeSize={measure}
          onscroll={measure}
          onwheel={onWheel}
        >
          <div class="relative" style:width="{layout.width}px" style:height="{thumbnailHeight}px">
            <div class="absolute top-0 flex gap-1" style:left="{view.offsetX}px">
              {#each visibleAssets as asset, visibleIndex (asset.id)}
                {@const box = layout.boxes[view.startIdx + visibleIndex] ?? { width: 0 }}
                {@const isCurrent = asset.id === currentId}
                {@const paneNumber = (highlightIds?.indexOf(asset.id) ?? -1) + 1}
                {@const isMember = !isCurrent && paneNumber > 0}
                {@const rating = asset.exifInfo?.rating ?? 0}
                {@const rejected = isRejected(asset)}
                {#if onSelect}
                  <ContextMenu disabled={!menu}>
                    {#snippet trigger(props)}
                      <button
                        {...mergeProps(props, {
                          onclick: (e: MouseEvent) =>
                            onSelect(asset.id, e.metaKey || e.ctrlKey || e.shiftKey)
                        })}
                        type="button"
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
                        {@render badges(
                          showBadges && asset.isFavorite,
                          showBadges && rejected,
                          labelOf(asset)
                        )}
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
                    {/snippet}
                    {@render menu?.(asset.id)}
                  </ContextMenu>
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
                    {@render badges(
                      showBadges && asset.isFavorite,
                      showBadges && rejected,
                      labelOf(asset)
                    )}
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
