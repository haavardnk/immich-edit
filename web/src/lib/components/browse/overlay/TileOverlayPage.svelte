<script lang="ts">
  import FilterPage from '#lib/components/browse/filters/FilterPage.svelte';
  import TileOverlayControls from '#lib/components/browse/overlay/TileOverlayControls.svelte';
  import TileOverlayReset from '#lib/components/browse/overlay/TileOverlayReset.svelte';
</script>

<FilterPage title="Thumbnail info">
  {#snippet action()}
    <TileOverlayReset />
  {/snippet}
  <TileOverlayControls />
</FilterPage>
