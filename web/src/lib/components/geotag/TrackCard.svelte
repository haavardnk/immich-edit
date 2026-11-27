<script lang="ts">
  import TrackPlaceButton from '#lib/components/geotag/TrackPlaceButton.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { trackSession } from '#lib/stores/trackSession.svelte.ts';

  let { height = $bindable(0) }: { height?: number } = $props();

  const session = trackSession;

  const column = $derived(layout.compact && !layout.landscape);
  const place = $derived(
    column
      ? 'inset-x-3 bottom-3 flex-col items-stretch gap-3 p-3.5'
      : layout.phone
        ? 'right-3 bottom-3 max-w-[calc(100%-1.5rem)] items-center gap-4 py-2.5 ps-4 pe-2.5'
        : 'bottom-6 left-1/2 max-w-[calc(100%-1.5rem)] -translate-x-1/2 items-center gap-4 py-2.5 ps-4 pe-2.5'
  );
</script>

<div
  class="absolute z-10 flex rounded-2xl border border-hairline bg-light-100 shadow-popover {place}"
  bind:offsetHeight={height}
  data-track-card
>
  <div class="min-w-0 flex-1 leading-tight">
    <p
      class="truncate font-semibold text-light-950 tabular-nums {column ? 'text-base' : 'text-sm'}"
    >
      {session.summary}
    </p>
    {#if session.shifts.status !== 'No change'}
      <p class="mt-0.5 text-dark/65 tabular-nums {column ? 'text-sm' : 'truncate text-caption'}">
        Camera time · {session.shifts.status}
      </p>
    {/if}
  </div>
  <TrackPlaceButton
    size={column ? 'large' : 'small'}
    class={column ? 'w-full' : 'shrink-0 whitespace-nowrap touch:h-11'}
  />
</div>
