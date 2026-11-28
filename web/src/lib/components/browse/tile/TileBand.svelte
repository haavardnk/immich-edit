<script lang="ts">
  import DisabledReason from '#lib/components/DisabledReason.svelte';
  import StarRating from '#lib/components/StarRating.svelte';
  import TileCluster from '#lib/components/browse/tile/TileCluster.svelte';
  import type { ClusterItem, TileDetail } from '#lib/browse/tileContent.ts';
  import type { TileLayout } from '#lib/browse/tileLayout.ts';
  import type { TileInfo } from '#lib/browse/tileOverlay.ts';
  import { NOT_OWNER } from '#lib/browse/ownership.ts';

  let {
    info,
    shape,
    details,
    cluster,
    rating,
    owned,
    onRate
  }: {
    info: TileInfo;
    shape: TileLayout;
    details: readonly TileDetail[];
    cluster: readonly ClusterItem[];
    rating: number;
    owned: boolean;
    onRate?: (rating: number | null) => void;
  } = $props();

  const hoverOnly = 'opacity-0 group-hover:opacity-100 group-has-[:focus-visible]:opacity-100';
  const hoverBlock = 'hidden group-hover:block group-has-[:focus-visible]:block';

  const starsPinned = $derived(shape.pinnedStars && onRate !== undefined);
  const items = $derived(starsPinned ? cluster.filter((item) => item.key !== 'rating') : cluster);
  const pinned = $derived(
    info === 'full'
      ? details.length > 0 || items.length > 0 || starsPinned
      : info === 'icons' && items.length > 0
  );
  const linesShown = $derived(info !== 'icons');
  const lineClass = $derived(linesShown ? '' : hoverBlock);
  const leading = $derived(details.slice(0, -1));
  const last = $derived(details.at(-1) ?? null);
  const lastLineCap = $derived(
    !shape.stacked && items.length > 0 ? 'max-w-[62%] shrink-0' : 'min-w-0 flex-1'
  );
  const ratingClass = $derived(
    onRate && !starsPinned ? 'group-hover:hidden group-has-[:focus-visible]:hidden' : ''
  );
</script>

{#snippet line(detail: TileDetail, first: boolean, extra: string)}
  <span
    data-testid={first ? 'tile-name' : undefined}
    class="truncate drop-shadow-md {first
      ? 'text-micro font-medium'
      : 'text-nano text-white/70'} {lineClass} {extra}"
  >
    {detail.text}
  </span>
{/snippet}

{#if details.length > 0 || items.length > 0 || onRate}
  <div
    data-testid="tile-info"
    class="pointer-events-none absolute inset-x-0 bottom-0 flex flex-col justify-end gap-0.5 bg-linear-to-t from-black/90 via-black/55 to-transparent text-white transition-opacity {shape.low
      ? 'px-1.25 pt-2.5 pb-1'
      : 'px-2 pt-4 pb-1.5'} {pinned ? 'opacity-100' : hoverOnly}"
  >
    {#if onRate}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <span
        class="pointer-events-auto -ml-1 self-start drop-shadow-md {starsPinned
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
    {#each leading as detail, i (detail.key)}
      {@render line(detail, i === 0, '')}
    {/each}
    {#if last || items.length > 0}
      <div
        class="flex min-h-3.5 gap-0.5 {shape.stacked
          ? 'flex-col items-stretch'
          : 'items-end gap-x-2'}"
      >
        {#if last}
          {@render line(last, leading.length === 0, lastLineCap)}
        {/if}
        {#if items.length > 0}
          <TileCluster {items} counts={shape.counts} {ratingClass} />
        {/if}
      </div>
    {/if}
  </div>
{/if}
