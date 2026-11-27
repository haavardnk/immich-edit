<script lang="ts">
  import FileImportButton from '#lib/components/FileImportButton.svelte';
  import MenuRow from '#lib/components/MenuRow.svelte';
  import Popover from '#lib/components/Popover.svelte';
  import TrackDisclosure from '#lib/components/geotag/TrackDisclosure.svelte';
  import ClockStepper from '#lib/components/metadata/ClockStepper.svelte';
  import { trackDetail, trackName } from '#lib/geotag/loadedTracks.ts';
  import { TRACK_EXTENSIONS } from '#lib/geotag/trackFile.ts';
  import { TOLERANCES, trackSession } from '#lib/stores/trackSession.svelte.ts';
  import { Button, IconButton } from '@immich/ui';
  import { mdiClose, mdiPlus } from '@mdi/js';

  const session = trackSession;

  let within = $state(false);
  let shown = $state<string | null>(null);

  const several = $derived(session.tracks.length > 1);

  function minutesLabel(minutes: number): string {
    return minutes === 60 ? '1 h' : `${minutes} min`;
  }
</script>

{#snippet details(name: string, detail: string)}
  <span class="block truncate text-xs font-semibold text-light-950 touch:text-sm">{name}</span>
  <span class="block truncate text-caption text-dark/65 tabular-nums">{detail}</span>
{/snippet}

<TrackDisclosure
  label="Track"
  title={session.tracks.map(trackName).join(', ')}
  caption={trackDetail(session.tracks, session.trackZone)}
>
  {#each session.tracks as track (track.id)}
    {@const expanded = !several || shown === track.id}
    {@const name = trackName(track)}
    {@const detail = trackDetail([track], session.trackZone)}
    <div class="rounded-lg bg-light-100 ring-1 ring-hairline" role="group" aria-label={name}>
      <div class="flex items-center gap-1 ps-3 pe-1">
        {#if several}
          <button
            type="button"
            class="min-w-0 flex-1 py-2 text-left leading-tight outline-none focus-visible:underline"
            aria-expanded={expanded}
            onclick={() => (shown = expanded ? null : track.id)}
          >
            {@render details(name, detail)}
          </button>
        {:else}
          <div class="min-w-0 flex-1 py-2 leading-tight">{@render details(name, detail)}</div>
        {/if}
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
      </div>
      {#if expanded}
        <div class="px-3 pb-3">
          <ClockStepper
            label="Shift by"
            name="Shift {name} by"
            value={track.offsetMs}
            onChange={(ms) => session.setTrackOffset(track.id, ms)}
          />
        </div>
      {/if}
    </div>
  {/each}
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
        class="self-start touch:h-11"
        leadingIcon={mdiPlus}
        disabled={busy || session.loading}
        onclick={open}
      >
        Add track file
      </Button>
    {/snippet}
  </FileImportButton>
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
