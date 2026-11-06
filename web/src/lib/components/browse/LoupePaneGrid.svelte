<script lang="ts">
  import { browsing } from '#lib/stores/browsing.svelte.ts';
  import { compare } from '#lib/stores/compare.svelte.ts';
  import type { View, ViewportMetrics } from '#lib/utils/imageViewport.ts';
  import { hint } from '#lib/shortcuts/labels.ts';
  import type { LoupeStage } from '#lib/browse/loupeStage.ts';
  import LoupePane from '#lib/components/browse/LoupePane.svelte';
  import LoupeSwipeTrack from '#lib/components/browse/LoupeSwipeTrack.svelte';
  import { IconButton } from '@immich/ui';
  import { mdiArrowCollapseLeft, mdiClose } from '@mdi/js';

  let {
    panes,
    focusedId,
    multi,
    canDrop,
    scrim,
    onView,
    onSize,
    onMetrics,
    onImage
  }: {
    panes: string[];
    focusedId: string | null;
    multi: boolean;
    canDrop: boolean;
    scrim: () => HTMLElement | null;
    onView: (id: string, view: View, solo?: boolean) => void;
    onSize: (maxEdge: number) => void;
    onMetrics: (id: string, metrics: ViewportMetrics) => void;
    onImage: (id: string, element: HTMLImageElement) => void;
  } = $props();
</script>

{#snippet pane(id: string, index: number, stage: LoupeStage | null)}
  {@const paneAsset = browsing.assets.find((item) => item.id === id)}
  <LoupePane
    assetId={id}
    alt={paneAsset?.originalFileName ?? ''}
    view={compare.viewOf(id)}
    focused={id === focusedId}
    showFocus={multi}
    badge={compare.mode === 'compare'
      ? index === 0
        ? 'Select'
        : 'Candidate'
      : multi
        ? String(index + 1)
        : undefined}
    onFocus={() => (compare.focusIndex = index)}
    onView={(next, solo) => onView(id, next, solo)}
    {onSize}
    sourceLong={Math.max(
      paneAsset?.exifInfo?.exifImageWidth ?? 0,
      paneAsset?.exifInfo?.exifImageHeight ?? 0
    )}
    onMetrics={(metrics) => onMetrics(id, metrics)}
    onImage={(element) => onImage(id, element)}
    {stage}
  />
{/snippet}

{#if multi}
  {#each panes as id, index (id)}
    <div class="group relative flex min-h-0 min-w-0 flex-1">
      {@render pane(id, index, null)}
      {#if compare.mode === 'compare' && index > 0 && id === focusedId}
        <IconButton
          type="button"
          size="small"
          variant="ghost"
          color="secondary"
          shape="round"
          class="absolute top-2 right-2 z-10 bg-black/50 text-white hover:bg-black/75"
          icon={mdiArrowCollapseLeft}
          title={hint('Make select', 'panePromote')}
          aria-label="Make select"
          onclick={() => compare.promote(index)}
        />
      {:else if compare.mode === 'survey' && canDrop}
        <IconButton
          type="button"
          size="small"
          variant="ghost"
          color="secondary"
          shape="round"
          class="absolute top-2 right-2 z-10 bg-black/50 text-white opacity-0 group-focus-within:opacity-100 group-hover:opacity-100 hover:bg-black/75 focus-visible:opacity-100"
          icon={mdiClose}
          title={hint('Drop this photo', 'paneDrop')}
          aria-label="Drop from survey"
          onclick={() => compare.drop(index)}
        />
      {/if}
    </div>
  {/each}
{:else if panes[0]}
  {@const id = panes[0]}
  <LoupeSwipeTrack {id} {scrim}>
    {#snippet children(stage)}
      {#key id}
        {@render pane(id, 0, stage)}
      {/key}
    {/snippet}
  </LoupeSwipeTrack>
{/if}
