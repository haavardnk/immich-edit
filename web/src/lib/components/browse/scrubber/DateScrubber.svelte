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

{#if scrub.model.months.length > 0}
  {#if layout.mode === 'touch'}
    <DateThumb {scrub} {scroller} />
  {:else}
    <DateRail {scrub} />
  {/if}
{/if}
