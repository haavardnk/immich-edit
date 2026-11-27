<script lang="ts">
  import { layout } from '#lib/device/layout.svelte.ts';
  import { trackSession } from '#lib/stores/trackSession.svelte.ts';
  import { IconButton } from '@immich/ui';
  import { mdiDockRight, mdiMapMarkerPath } from '@mdi/js';

  let { panel, onPanel }: { panel: boolean; onPanel: () => void } = $props();

  const tracking = $derived(trackSession.active);
</script>

<IconButton
  size="small"
  variant="ghost"
  color={tracking ? 'primary' : 'secondary'}
  class="touch:size-11"
  icon={mdiMapMarkerPath}
  title={tracking ? 'Close track' : 'Geotag photos from a GPS track'}
  aria-label="Geotag from track"
  aria-pressed={tracking}
  onclick={() => (tracking ? trackSession.close() : trackSession.open())}
/>
{#if !layout.compact}
  <IconButton
    size="small"
    variant="ghost"
    color={panel ? 'primary' : 'secondary'}
    class="touch:size-11"
    icon={mdiDockRight}
    title={panel ? 'Hide side panel' : 'Show side panel'}
    aria-label="Side panel"
    aria-pressed={panel}
    onclick={onPanel}
  />
{/if}
