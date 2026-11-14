<script lang="ts">
  import FilterToggle from '#lib/components/browse/FilterToggle.svelte';
  import SelectModeButton from '#lib/components/browse/SelectModeButton.svelte';
  import { VIEW_AND_FILTERS } from '#lib/browse/filterSections.ts';
</script>

<FilterToggle label={VIEW_AND_FILTERS} />
<SelectModeButton />
