<script lang="ts">
  import { browsing } from '$lib/stores/browsing.svelte';
  import { compare, type PaneView } from '$lib/stores/compare.svelte';
  import { hint } from '$lib/shortcuts/labels';
  import LoupePane from '$lib/components/browse/LoupePane.svelte';
  import { IconButton } from '@immich/ui';
  import { mdiArrowCollapseLeft, mdiClose } from '@mdi/js';

  let {
    panes,
    focusedId,
    multi,
    canDrop,
    onView,
    onSize,
    onFitZoom,
    onImage
  }: {
    panes: string[];
    focusedId: string | null;
    multi: boolean;
    canDrop: boolean;
    onView: (id: string, view: PaneView, solo?: boolean) => void;
    onSize: (maxEdge: number) => void;
    onFitZoom: (id: string, value: number) => void;
    onImage: (id: string, element: HTMLImageElement) => void;
  } = $props();

  let paneRefs = $state<Record<string, ReturnType<typeof LoupePane>>>({});

  export function panBy(id: string, fx: number, fy: number): void {
    paneRefs[id]?.panBy(fx, fy);
  }
</script>

{#each panes as id, index (id)}
  {@const paneAsset = browsing.assets.find((item) => item.id === id)}
  <div class="group relative flex min-h-0 min-w-0 flex-1">
    <LoupePane
      bind:this={paneRefs[id]}
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
      onFitZoom={(value) => onFitZoom(id, value)}
      onImage={(element) => onImage(id, element)}
    />
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
