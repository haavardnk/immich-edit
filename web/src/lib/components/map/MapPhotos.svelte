<script lang="ts">
  import AssetGrid from '#lib/components/browse/AssetGrid.svelte';
  import SegmentedChoice from '#lib/components/settings/SegmentedChoice.svelte';
  import type { MapPageState, MapTab } from '#lib/map/mapPage.svelte.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { Button } from '@immich/ui';

  let { map, heading = true }: { map: MapPageState; heading?: boolean } = $props();

  const TABS = [
    { value: 'view', label: 'In view' },
    { value: 'unplaced', label: 'No location' }
  ] as const satisfies readonly { value: MapTab; label: string }[];

  const unplaced = $derived(map.tab === 'unplaced');
  const feed = $derived(unplaced ? map.unplaced : map.feed);
  const total = $derived(unplaced ? undefined : map.shownIds.length);
  const empty = $derived(
    unplaced ? map.unplaced.loaded && map.unplaced.assets.length === 0 : total === 0 && map.loaded
  );
  const hint = $derived(
    layout.mode === 'touch'
      ? 'Select photos and use Location to place them'
      : 'Drag photos onto the map to place them'
  );
</script>

{#if heading}
  <div class="flex h-13 shrink-0 items-center gap-2 border-b border-hairline ps-4 pe-2">
    <div class="min-w-0 flex-1 leading-tight">
      <h2 class="truncate text-sm font-semibold text-light-950">{map.title}</h2>
      {#if unplaced}
        <p class="truncate text-caption text-dark/65">{hint}</p>
      {:else if map.place}
        <p class="truncate text-caption text-dark/65">{map.place}</p>
      {/if}
    </div>
    {#if map.picked && !unplaced}
      <Button size="tiny" variant="ghost" onclick={() => map.showAll()}>Show all in view</Button>
    {/if}
  </div>
{/if}
<div class="shrink-0 px-3 pt-2 pb-1">
  <SegmentedChoice
    options={TABS}
    value={map.tab}
    label="Photos"
    appearance="view"
    fill
    onChange={(tab) => (map.tab = tab)}
  />
</div>
<div class="min-h-0 flex-1 overflow-y-auto scrollbar-hidden">
  {#if feed.error}
    <p class="px-6 py-10 text-center text-sm text-danger" role="alert">{feed.error}</p>
  {:else if empty}
    <p class="px-6 py-10 text-center text-sm text-dark/65" role="status">
      {unplaced
        ? 'Every photo matching these filters has a location.'
        : 'No photos here. Zoom out or pan to see more.'}
    </p>
  {:else}
    <AssetGrid
      assets={feed.assets}
      {total}
      loadingMore={feed.loadingMore}
      dayHeaders
      rowHeight={layout.mode === 'touch' ? 120 : 104}
      onLoadMore={() => void feed.loadMore()}
    />
  {/if}
</div>
