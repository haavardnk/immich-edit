<script lang="ts">
  import TrackFitBar from '#lib/components/geotag/TrackFitBar.svelte';
  import TrackSetup from '#lib/components/geotag/TrackSetup.svelte';
  import TrackFooter from '#lib/components/geotag/TrackFooter.svelte';
  import TrackPhotoList from '#lib/components/geotag/TrackPhotoList.svelte';
  import TrackSources from '#lib/components/geotag/TrackSources.svelte';
  import { trackSession } from '#lib/stores/trackSession.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { IconButton } from '@immich/ui';
  import { mdiClose } from '@mdi/js';

  let { header = true, footer = true }: { header?: boolean; footer?: boolean } = $props();

  const session = trackSession;

  const subtitle = $derived(
    session.scope === 'selection'
      ? countOf(session.photos.length, 'selected photo')
      : session.tracks.length > 0
        ? `${countOf(session.photos.length, 'photo')} taken during the track`
        : 'Photos taken during the track'
  );
</script>

<div class="flex h-full min-h-0 flex-col" data-track-panel>
  {#if header}
    <div class="flex h-13 shrink-0 items-center gap-2 border-b border-hairline ps-4 pe-2">
      <div class="min-w-0 flex-1 leading-tight">
        <h2 class="truncate text-sm font-semibold text-light-950">Match track</h2>
        <p class="truncate text-caption text-dark/65">{subtitle}</p>
      </div>
      <IconButton
        size="small"
        variant="ghost"
        color="secondary"
        class="touch:size-11"
        icon={mdiClose}
        title="Close track"
        aria-label="Close track"
        onclick={session.close}
      />
    </div>
  {/if}
  {#if session.tracks.length > 0}
    <div class="flex min-h-0 flex-1 flex-col overflow-y-auto">
      <div class="shrink-0 border-b border-hairline">
        <TrackSetup />
        <TrackFitBar />
      </div>
      <TrackPhotoList />
    </div>
  {:else}
    <div class="min-h-0 flex-1 overflow-y-auto">
      <TrackSources />
    </div>
  {/if}
  {#if footer && session.tracks.length > 0}
    <TrackFooter />
  {/if}
</div>
