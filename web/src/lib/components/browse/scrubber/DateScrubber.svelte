<script lang="ts">
  import { DateScrub } from '#lib/browse/dateScrub.svelte.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
  import type { BrowseFeed } from '#lib/stores/browseFeed.svelte.ts';
  import { monthCounts } from '#lib/stores/monthCounts.svelte.ts';
  import DateRail from '#lib/components/browse/scrubber/DateRail.svelte';
  import DateThumb from '#lib/components/browse/scrubber/DateThumb.svelte';

  let { feed, scroller }: { feed: BrowseFeed; scroller: HTMLElement | undefined } = $props();

  const scrub = new DateScrub(() => feed);

  $effect(() => monthCounts.watch(feed.currentFilter()));
</script>

{#if layout.mode === 'touch'}
  {#if scrub.model.months.length > 0}
    <DateThumb {scrub} {scroller} />
  {/if}
{:else if scrub.model.months.length > 0 || !monthCounts.loaded}
  <div class="flex w-13 shrink-0" data-testid="date-rail-slot">
    {#if scrub.model.months.length > 0}
      <DateRail {scrub} />
    {/if}
  </div>
{/if}
