<script lang="ts">
  import { DEFAULT_TILE_OVERLAY, isDefaultOverlay } from '#lib/browse/tileOverlay.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { preferences } from '#lib/stores/preferences.svelte.ts';
  import { Button } from '@immich/ui';
  import { mdiRestore } from '@mdi/js';
</script>

<Button
  size="tiny"
  variant="ghost"
  color={layout.mode === 'touch' ? 'primary' : 'secondary'}
  class="ml-auto h-8 shrink-0 bg-transparent hover:bg-light-950/6 touch:text-label"
  leadingIcon={mdiRestore}
  aria-label="Reset thumbnail info"
  disabled={isDefaultOverlay(preferences.overlay)}
  onclick={() => void preferences.setTileOverlay(DEFAULT_TILE_OVERLAY)}
>
  Reset
</Button>
