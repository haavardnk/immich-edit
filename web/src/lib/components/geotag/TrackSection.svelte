<script lang="ts">
  import FileImportButton from '#lib/components/FileImportButton.svelte';
  import MenuRow from '#lib/components/MenuRow.svelte';
  import Popover from '#lib/components/Popover.svelte';
  import TrackClockFields from '#lib/components/geotag/TrackClockFields.svelte';
  import TrackDisclosure from '#lib/components/geotag/TrackDisclosure.svelte';
  import TrackExpandRow from '#lib/components/geotag/TrackExpandRow.svelte';
  import {
    trackChangeText,
    trackDetail,
    trackName,
    type LoadedTrack
  } from '#lib/geotag/loadedTracks.ts';
  import { TRACK_EXTENSIONS } from '#lib/geotag/trackFile.ts';
  import { TOLERANCES, trackSession } from '#lib/stores/trackSession.svelte.ts';
  import { Button, IconButton } from '@immich/ui';
  import { mdiClose, mdiPlus } from '@mdi/js';

  const AS_FILE = 'as-file';

  const session = trackSession;

  let within = $state(false);
  let shown = $state<string | null>(null);

  const only = $derived(session.tracks.length === 1 ? session.tracks[0] : undefined);

  function minutesLabel(minutes: number): string {
    return minutes === 60 ? '1 h' : `${minutes} min`;
  }
</script>

{#snippet fields(track: LoadedTrack, width?: string)}
  <TrackClockFields
    {width}
    name={trackName(track)}
    zone={track.zone ?? AS_FILE}
    zones={track.utc}
    at={track.start}
    choices={[{ id: AS_FILE, label: 'UTC' }]}
    onZone={(id) => session.setTrackZone(track.id, id === AS_FILE ? null : id)}
    shiftMs={track.offsetMs}
    onShift={(ms) => session.setTrackOffset(track.id, ms)}
  />
{/snippet}

<TrackDisclosure
  label="Track"
  title={session.tracks.map(trackName).join(', ')}
  caption={trackDetail(session.tracks, session.trackZone)}
>
  {#if only}
    {@render fields(only)}
  {:else}
    {#each session.tracks as track (track.id)}
      <TrackExpandRow
        label={trackName(track)}
        value={trackChangeText(track)}
        open={shown === track.id}
        onToggle={() => (shown = shown === track.id ? null : track.id)}
      >
        {#snippet trailing()}
          <IconButton
            size="small"
            variant="ghost"
            color="secondary"
            shape="round"
            class="touch:size-11"
            icon={mdiClose}
            title="Remove {track.fileName}"
            aria-label="Remove {track.fileName}"
            onclick={() => session.removeTrack(track.id)}
          />
        {/snippet}
        {@render fields(track, 'w-52')}
      </TrackExpandRow>
    {/each}
  {/if}
  <div class="flex items-center gap-2 [&>*]:flex-1">
    <FileImportButton
      accept={TRACK_EXTENSIONS.join(',')}
      label="Add track file"
      multiple
      onfile={(file) => session.addFiles([file])}
    >
      {#snippet button({ busy, open })}
        <Button
          size="small"
          variant="ghost"
          class="touch:h-11"
          leadingIcon={mdiPlus}
          disabled={busy || session.loading}
          onclick={open}
        >
          Add track file
        </Button>
      {/snippet}
    </FileImportButton>
    {#if only}
      <Button
        size="small"
        variant="ghost"
        color="secondary"
        class="touch:h-11"
        leadingIcon={mdiClose}
        aria-label="Remove {only.fileName}"
        onclick={() => session.removeTrack(only.id)}
      >
        Remove track
      </Button>
    {/if}
  </div>
  <div class="flex items-center gap-3">
    <span class="min-w-0 flex-1 text-sm text-light-950">Match within</span>
    <Popover
      open={within}
      onOpenChange={(open) => (within = open)}
      title="Match within"
      subtitle="How far from a track point a photo may be"
    >
      {#snippet trigger(props)}
        <Button
          {...props}
          size="small"
          variant="ghost"
          color="secondary"
          class="h-8 bg-light-100 ring-1 ring-hairline touch:h-11"
          aria-label="Match within {minutesLabel(session.toleranceMin)}"
        >
          {minutesLabel(session.toleranceMin)}
        </Button>
      {/snippet}
      {#each TOLERANCES as minutes (minutes)}
        <MenuRow
          label={minutesLabel(minutes)}
          pressed={session.toleranceMin === minutes}
          onclick={() => {
            session.toleranceMin = minutes;
            within = false;
          }}
        />
      {/each}
    </Popover>
  </div>
</TrackDisclosure>
