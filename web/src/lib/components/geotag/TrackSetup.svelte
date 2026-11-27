<script lang="ts">
  import TrackDisclosure from '#lib/components/geotag/TrackDisclosure.svelte';
  import TrackSettings from '#lib/components/geotag/TrackSettings.svelte';
  import { trackSession } from '#lib/stores/trackSession.svelte.ts';

  const session = trackSession;
</script>

<TrackDisclosure
  label="Clock setup"
  title="Camera clock"
  caption="{session.clockText} · {session.zone.replaceAll('_', ' ')}"
>
  <TrackSettings />
</TrackDisclosure>
