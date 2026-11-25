<script lang="ts">
  import TrackFiles from '#lib/components/geotag/TrackFiles.svelte';
  import TrackSettings from '#lib/components/geotag/TrackSettings.svelte';
  import { trackSession } from '#lib/stores/trackSession.svelte.ts';
  import { Icon } from '@immich/ui';
  import { mdiChevronDown } from '@mdi/js';

  const session = trackSession;

  let open = $state(false);

  const names = $derived(session.tracks.map((track) => track.name ?? track.fileName).join(', '));
</script>

<div class="flex flex-col">
  <button
    type="button"
    class="flex w-full items-center gap-3 py-2 ps-4 pe-3 text-left transition-colors outline-none hover:bg-light-100 focus-visible:bg-light-100 touch:min-h-13"
    aria-expanded={open}
    aria-controls="track-setup"
    onclick={() => (open = !open)}
  >
    <span class="min-w-0 flex-1 leading-tight">
      <span class="block truncate text-sm text-light-950">{names}</span>
      <span class="block truncate text-caption text-dark/65">
        <span class="sr-only">Camera clock:</span>
        {session.clockText} · {session.zone.replaceAll('_', ' ')}
      </span>
    </span>
    <Icon
      icon={mdiChevronDown}
      size="16px"
      class="shrink-0 text-dark/65 transition-transform {open ? 'rotate-180' : ''}"
      aria-hidden
    />
  </button>
  {#if open}
    <div id="track-setup" class="pb-1">
      <TrackFiles named={false} />
      <TrackSettings />
    </div>
  {/if}
</div>
