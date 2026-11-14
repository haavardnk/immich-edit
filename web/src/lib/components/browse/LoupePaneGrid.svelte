<script lang="ts">
  import { browsing } from '#lib/stores/browsing.svelte.ts';
  import { compare } from '#lib/stores/compare.svelte.ts';
  import type { View, ViewportMetrics } from '#lib/utils/imageViewport.ts';
  import type { LoupeStage } from '#lib/browse/loupeStage.ts';
  import LoupePane from '#lib/components/browse/LoupePane.svelte';
  import LoupeSwipeTrack from '#lib/components/browse/LoupeSwipeTrack.svelte';
  import { isRejected } from '#lib/browse/reject.ts';
  import { paneDrop, panePromote } from '#lib/commands/loupe/panes.ts';
  import type { IconCommand } from '#lib/commands/types.ts';
  import CommandIconButton from '#lib/components/CommandIconButton.svelte';
  import CommandPill from '#lib/components/CommandPill.svelte';

  let {
    panes,
    focusedId,
    first,
    multi,
    paneActions,
    pills,
    scrim,
    onView,
    onSize,
    onMetrics,
    onImage
  }: {
    panes: string[];
    focusedId: string | null;
    first: number;
    multi: boolean;
    paneActions: readonly IconCommand[];
    pills: boolean;
    scrim: () => HTMLElement | null;
    onView: (id: string, view: View, solo?: boolean) => void;
    onSize: (maxEdge: number) => void;
    onMetrics: (id: string, metrics: ViewportMetrics) => void;
    onImage: (id: string, element: HTMLImageElement) => void;
  } = $props();

  const onPhoto = 'absolute top-2 right-2 z-10 bg-black/50 text-white hover:bg-black/75';
</script>

{#snippet pane(id: string, index: number, stage: LoupeStage | null)}
  {@const paneAsset = browsing.assets.find((item) => item.id === id)}
  <LoupePane
    assetId={id}
    thumbhash={paneAsset?.thumbhash}
    checksum={paneAsset?.checksum}
    alt={paneAsset?.originalFileName ?? ''}
    view={compare.viewOf(id)}
    focused={id === focusedId}
    showFocus={multi}
    badge={multi ? String(index + 1) : undefined}
    caption={compare.mode === 'compare' ? (index === 0 ? 'Select' : 'Candidate') : undefined}
    rejected={multi && paneAsset !== undefined && isRejected(paneAsset)}
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
  {#each panes as id, offset (id)}
    {@const index = first + offset}
    <div class="group relative flex min-h-0 min-w-0 flex-1">
      {@render pane(id, index, null)}
      {#if compare.mode === 'compare' && index > 0 && id === focusedId && paneActions.includes(panePromote)}
        {#if pills}
          <CommandPill command={panePromote} long class="{onPhoto} h-11 gap-1.5 px-3.5" />
        {:else}
          <CommandIconButton
            command={panePromote}
            size="small"
            variant="ghost"
            shape="round"
            class={onPhoto}
          />
        {/if}
      {:else if compare.mode === 'survey' && paneActions.includes(paneDrop)}
        <CommandIconButton
          command={paneDrop}
          before={() => (compare.focusIndex = index)}
          size="small"
          variant="ghost"
          shape="round"
          class="{onPhoto} opacity-0 group-focus-within:opacity-100 group-hover:opacity-100 focus-visible:opacity-100"
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
