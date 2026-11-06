<script lang="ts">
  import type { GridMarquee } from '#lib/browse/gridMarquee.svelte.ts';

  let { marquee }: { marquee: GridMarquee } = $props();
</script>

{#if marquee.active}
  <div
    {@attach marquee.boxRef}
    data-testid="marquee"
    aria-hidden="true"
    class="pointer-events-none absolute top-0 left-0 z-20 rounded-sm border border-primary bg-primary/10"
  ></div>
  <div
    {@attach marquee.chipRef}
    data-testid="marquee-count"
    aria-hidden="true"
    class="pointer-events-none absolute top-0 left-0 z-30 w-max min-w-7 rounded-full bg-light-50/95 px-2 py-0.5 text-center text-xs font-semibold tabular-nums text-light-950 shadow-popover ring-1 ring-hairline backdrop-blur-md"
  >
    {marquee.adding ? '+' : ''}{marquee.covered}
  </div>
{/if}
