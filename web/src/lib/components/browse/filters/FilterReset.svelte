<script lang="ts">
  import { activeFilterCount } from '#lib/browse/filterChips.ts';
  import PanelReset from '#lib/components/browse/PanelReset.svelte';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
</script>

<PanelReset
  label="Reset all filters"
  disabled={activeFilterCount(browseControls.filters) === 0}
  onclick={() => browseControls.resetFilters()}
/>
