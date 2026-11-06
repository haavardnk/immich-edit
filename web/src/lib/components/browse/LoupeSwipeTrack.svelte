<script lang="ts">
  import { immichPreviewUrl } from '#lib/api/assets.ts';
  import { canStepLoupe, stepLoupe } from '#lib/browse/loupeNav.ts';
  import { LoupeStage } from '#lib/browse/loupeStage.ts';
  import { SWIPE_GAP } from '#lib/browse/loupeSwipe.ts';
  import { runCommand } from '#lib/commands/command.ts';
  import { loupeClose } from '#lib/commands/loupe/dismiss.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';
  import { browsing } from '#lib/stores/browsing.svelte.ts';
  import { onDestroy, type Snippet } from 'svelte';
  import { prefersReducedMotion } from 'svelte/motion';

  let {
    id,
    scrim,
    children
  }: {
    id: string;
    scrim: () => HTMLElement | null;
    children: Snippet<[LoupeStage]>;
  } = $props();

  let track = $state<HTMLDivElement | null>(null);

  const neighbours = $derived([
    { side: -1, asset: browsing.prevOf(id) },
    { side: 1, asset: browsing.nextOf(id) }
  ]);

  const stage = new LoupeStage({
    track: () => track,
    scrim: () => scrim(),
    canStep: (delta) => canStepLoupe(id, delta),
    hasNeighbour: (delta) => neighbours.some((item) => item.side === delta && item.asset !== null),
    current: () => browseView.loupeId,
    step: stepLoupe,
    dismiss: () => runCommand(loupeClose),
    openInfo: () => browseView.setLoupeInfoOpen(true),
    reducedMotion: () => prefersReducedMotion.current
  });

  onDestroy(() => stage.dispose());
</script>

<div class="relative flex min-h-0 min-w-0 flex-1 overflow-hidden">
  <div bind:this={track} data-testid="loupe-track" class="relative flex min-h-0 min-w-0 flex-1">
    {#each neighbours as neighbour (neighbour.side)}
      {#if neighbour.asset}
        <img
          src={immichPreviewUrl(neighbour.asset)}
          alt=""
          aria-hidden="true"
          draggable="false"
          data-testid="loupe-neighbour"
          class="pointer-events-none absolute inset-0 h-full w-full object-contain select-none"
          style="translate: calc({neighbour.side} * (100% + {SWIPE_GAP}px)) 0"
        />
      {/if}
    {/each}
    {@render children(stage)}
  </div>
</div>
