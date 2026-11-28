<script lang="ts">
  import { DEFAULT_TILE_OVERLAY, isDefaultOverlay } from '#lib/browse/tileOverlay.ts';
  import PanelReset from '#lib/components/browse/PanelReset.svelte';
  import { preferences } from '#lib/stores/preferences.svelte.ts';
</script>

<PanelReset
  label="Reset thumbnail info"
  disabled={isDefaultOverlay(preferences.overlay)}
  onclick={() => void preferences.setTileOverlay(DEFAULT_TILE_OVERLAY)}
/>
