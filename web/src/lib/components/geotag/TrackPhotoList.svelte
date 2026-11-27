<script lang="ts">
  import { thumbUrl } from '#lib/api/assets.ts';
  import TooltipLink from '#lib/components/TooltipLink.svelte';
  import Chip from '#lib/components/chips/Chip.svelte';
  import { editorHref } from '#lib/editor/navigation.ts';
  import { openTrackPhoto } from '#lib/geotag/trackLoupe.ts';
  import { MATCH_TONES, matchStatus, matchTimes } from '#lib/geotag/trackRows.ts';
  import { trackSession } from '#lib/stores/trackSession.svelte.ts';
  import { currentPath } from '#lib/utils/route.ts';
  import { centerWithin, scrollParent } from '#lib/utils/scrollReveal.ts';

  type Filter = 'placed' | 'outside' | 'kept';

  const session = trackSession;

  let filter = $state<Filter | null>(null);
  let list = $state<HTMLElement | null>(null);

  const rows = $derived(
    session.rows.filter(
      ({ match }) =>
        !filter ||
        match.outcome.kind === filter ||
        (filter === 'outside' && match.outcome.kind === 'untimed')
    )
  );

  $effect(() => {
    const id = session.revealed;
    if (!id || !list) return;
    const row = list.querySelector<HTMLElement>(`[data-track-photo="${CSS.escape(id)}"]`);
    const scroller = row ? scrollParent(row) : null;
    if (row && scroller) centerWithin(row, scroller);
  });

  function open(event: MouseEvent, id: string): void {
    event.preventDefault();
    openTrackPhoto(id);
  }

  const chips: { id: Filter; label: string }[] = $derived([
    { id: 'placed', label: `${session.counts.placed} on track` },
    ...(session.counts.outside + session.counts.untimed > 0
      ? [
          {
            id: 'outside' as const,
            label: `${session.counts.outside + session.counts.untimed} outside`
          }
        ]
      : []),
    ...(session.counts.kept > 0
      ? [{ id: 'kept' as const, label: `${session.counts.kept} with location` }]
      : [])
  ]);
</script>

<section class="flex min-h-40 flex-1 flex-col" aria-label="Photos">
  <div class="flex shrink-0 items-center gap-1.5 px-4 pt-3 pb-2">
    <h3 class="section-label me-2 shrink-0 text-dark/65 touch:sr-only">Photos</h3>
    {#each chips as chip (chip.id)}
      <Chip
        class="touch:min-w-0 touch:grow touch:px-2.5 {chip.id === 'kept' ? 'touch:shrink' : ''}"
        pressed={filter === chip.id}
        onPressedChange={(on) => (filter = on ? chip.id : null)}
      >
        <span class="min-w-0 truncate">{chip.label}</span>
      </Chip>
    {/each}
  </div>
  <ul
    class="flex min-h-0 flex-1 flex-col overflow-y-auto pb-2"
    aria-label="Photos on the track"
    bind:this={list}
  >
    {#each rows as { asset, match } (asset.id)}
      {@const state = matchStatus(match)}
      {@const when = matchTimes(session.bases.get(asset.id), match, session.trackZone)}
      <li
        class="flex items-center gap-3 px-4 py-1.5 transition-colors [contain-intrinsic-size:auto_3.25rem] [content-visibility:auto] touch:[contain-intrinsic-size:auto_3.75rem] {session.focused ===
        asset.id
          ? 'bg-primary/20 shadow-[inset_3px_0_0_var(--color-primary)]'
          : ''}"
        data-track-photo={asset.id}
        aria-current={session.focused === asset.id ? 'true' : undefined}
        onpointerenter={() => (session.focused = asset.id)}
        onpointerleave={() => (session.focused = null)}
      >
        <TooltipLink
          tooltip={asset.originalFileName}
          href={editorHref(asset.id, currentPath())}
          onclick={(event) => open(event, asset.id)}
          aria-label="Open {asset.originalFileName}"
          class="flex min-w-0 flex-1 items-center gap-3 rounded outline-none focus-visible:ring-2 focus-visible:ring-primary"
        >
          <img
            src={thumbUrl(asset.id, 'thumbnail')}
            alt=""
            loading="lazy"
            class="size-10 shrink-0 rounded object-cover touch:size-12"
          />
          <span class="min-w-0 flex-1">
            <span class="block truncate text-xs text-light-950 touch:text-sm">
              {asset.originalFileName}
            </span>
            {#if when}
              <span class="block truncate text-caption text-dark/65 tabular-nums">{when}</span>
            {/if}
          </span>
        </TooltipLink>
        {#if state.text}
          <span class="shrink-0 rounded-full px-2 py-0.5 text-caption {MATCH_TONES[state.tone]}">
            {state.text}
          </span>
        {/if}
      </li>
    {:else}
      <li class="px-4 py-3 text-caption text-dark/65">
        {session.tracks.length === 0
          ? 'Load a track to see where your photos land'
          : 'No photos were taken during the track'}
      </li>
    {/each}
  </ul>
</section>
