<script lang="ts">
  import { thumbUrl } from '#lib/api/assets.ts';
  import TooltipLink from '#lib/components/TooltipLink.svelte';
  import { editorHref } from '#lib/editor/navigation.ts';
  import { currentPath } from '#lib/utils/route.ts';
  import { MATCH_TONES, matchStatus, matchTimes } from '#lib/geotag/trackRows.ts';
  import type { PinPoint } from '#lib/map/trackOverlay.ts';
  import { trackSession } from '#lib/stores/trackSession.svelte.ts';

  let {
    id,
    at,
    compact = false,
    onOpen
  }: { id: string; at: PinPoint; compact?: boolean; onOpen?: () => void } = $props();

  const CARD_WIDTH = 240;
  const CARD_HEIGHT = 228;
  const COMPACT_WIDTH = 280;
  const COMPACT_HEIGHT = 92;
  const GAP = 18;

  const row = $derived(trackSession.row(id));
  const beside = $derived(
    !compact && (at.x + GAP + CARD_WIDTH <= at.width || at.x - GAP - CARD_WIDTH >= 0)
  );
  const left = $derived.by(() => {
    if (!beside) {
      return Math.min(Math.max(8, at.x - COMPACT_WIDTH / 2), at.width - COMPACT_WIDTH - 8);
    }
    return at.x + GAP + CARD_WIDTH <= at.width ? at.x + GAP : at.x - GAP - CARD_WIDTH;
  });
  const top = $derived.by(() => {
    if (!beside) {
      return at.y - GAP - COMPACT_HEIGHT >= 8 ? at.y - GAP - COMPACT_HEIGHT : at.y + GAP;
    }
    return Math.min(Math.max(8, at.y - CARD_HEIGHT / 2), at.height - CARD_HEIGHT - 8);
  });
</script>

{#if row}
  {@const state = matchStatus(row.match)}
  {@const when = matchTimes(trackSession.bases.get(id), row.match, trackSession.zone)}
  <div
    class="absolute z-30 flex overflow-hidden rounded-xl bg-light-100 shadow-popover ring-1 ring-hairline {beside
      ? 'w-60 flex-col'
      : 'w-70 items-center gap-3 p-2.5'}"
    class:pointer-events-none={!onOpen}
    style:left="{left}px"
    style:top="{top}px"
    data-pin-preview
  >
    {#if onOpen}
      <TooltipLink
        tooltip="Open {row.asset.originalFileName}"
        href={editorHref(id, currentPath())}
        onclick={(event) => {
          event.preventDefault();
          onOpen();
        }}
        aria-label="Open {row.asset.originalFileName}"
        class="absolute inset-0 z-10 rounded-xl outline-none focus-visible:ring-2 focus-visible:ring-primary"
      >
        <span class="sr-only">Open</span>
      </TooltipLink>
    {/if}
    <img
      src={thumbUrl(id, 'preview')}
      alt={row.asset.originalFileName}
      class="bg-light-200 object-cover {beside ? 'h-40 w-full' : 'size-18 shrink-0 rounded-md'}"
    />
    <div class="flex min-w-0 flex-1 flex-col gap-1 {beside ? 'px-3 py-2' : ''}">
      <p class="truncate text-xs font-semibold text-light-950">{row.asset.originalFileName}</p>
      <div class="flex items-center gap-2">
        {#if when}
          <span class="min-w-0 flex-1 truncate text-caption text-dark/65 tabular-nums">{when}</span>
        {/if}
        <span
          class="ms-auto shrink-0 rounded-full px-2 py-0.5 text-caption {MATCH_TONES[state.tone]}"
        >
          {state.text}
        </span>
      </div>
    </div>
  </div>
{/if}
