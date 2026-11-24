<script lang="ts">
  import FileImportButton from '#lib/components/FileImportButton.svelte';
  import MenuRow from '#lib/components/MenuRow.svelte';
  import MoreActionsButton from '#lib/components/MoreActionsButton.svelte';
  import { TRACK_EXTENSIONS } from '#lib/geotag/trackFile.ts';
  import { trackSession } from '#lib/stores/trackSession.svelte.ts';
  import { IconButton } from '@immich/ui';
  import { mdiClose, mdiFilePlusOutline } from '@mdi/js';

  let { named = true }: { named?: boolean } = $props();

  const session = trackSession;

  const names = $derived(session.tracks.map((track) => track.name ?? track.fileName).join(', '));
  const detail = $derived.by(() => {
    const tracks = session.tracks;
    if (tracks.length === 0) return '';
    const start = Math.min(...tracks.map((track) => track.start));
    const end = Math.max(...tracks.map((track) => track.end));
    const metres = tracks.reduce((sum, track) => sum + track.metres, 0);
    return `${dateRange(start, end)} · ${distance(metres)}`;
  });

  function dateRange(start: number, end: number): string {
    const day = new Intl.DateTimeFormat(undefined, { day: 'numeric', month: 'short' });
    const time = new Intl.DateTimeFormat(undefined, { hour: '2-digit', minute: '2-digit' });
    return `${day.format(start)} · ${time.format(start)}–${time.format(end)}`;
  }

  function distance(metres: number): string {
    return metres < 1000 ? `${Math.round(metres)} m` : `${(metres / 1000).toFixed(1)} km`;
  }
</script>

<div class="flex items-center gap-1 py-1 ps-4 pe-2" role="group" aria-label="Track">
  <div class="min-w-0 flex-1 leading-tight">
    {#if named}
      <p class="truncate text-sm text-light-950">{names}</p>
    {/if}
    <p class="truncate text-caption text-dark/65 tabular-nums">{detail}</p>
  </div>
  <FileImportButton
    accept={TRACK_EXTENSIONS.join(',')}
    label="Add track file"
    multiple
    onfile={(file) => session.addFiles([file])}
  >
    {#snippet button({ busy, open })}
      <IconButton
        size="small"
        variant="ghost"
        color="secondary"
        shape="round"
        class="touch:size-11"
        icon={mdiFilePlusOutline}
        title="Add track file"
        aria-label="Add track file"
        disabled={busy || session.loading}
        onclick={open}
      />
    {/snippet}
  </FileImportButton>
  <MoreActionsButton label="Track files" title="Track files">
    {#snippet children(close)}
      {#each session.tracks as track (track.id)}
        <MenuRow
          icon={mdiClose}
          label="Remove {track.fileName}"
          onclick={() => {
            close();
            session.removeTrack(track.id);
          }}
        />
      {/each}
    {/snippet}
  </MoreActionsButton>
</div>
