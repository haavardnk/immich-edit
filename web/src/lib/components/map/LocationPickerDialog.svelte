<script lang="ts">
  import LocationPicker from '#lib/components/map/LocationPicker.svelte';
  import { locationPicker } from '#lib/stores/locationPicker.svelte.ts';
</script>

{#if locationPicker.request}
  {#key locationPicker.request}
    <LocationPicker request={locationPicker.request} />
  {/key}
{/if}
