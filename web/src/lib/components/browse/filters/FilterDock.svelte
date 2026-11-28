<script lang="ts">
  import { activeFilterCount } from '#lib/browse/filterChips.ts';
  import DockPanel from '#lib/components/browse/DockPanel.svelte';
  import FilterCount from '#lib/components/browse/filters/FilterCount.svelte';
  import FilterReset from '#lib/components/browse/filters/FilterReset.svelte';
  import FilterSections from '#lib/components/browse/filters/FilterSections.svelte';
  import SaveViewScope from '#lib/components/savedViews/SaveViewScope.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { hint } from '#lib/shortcuts/labels.ts';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';

  let {
    favoriteLocked,
    title,
    onClose
  }: { favoriteLocked: boolean; title: string; onClose: () => void } = $props();
</script>

<SaveViewScope {title}>
  {#snippet children(saveButton, saveEntry)}
    <DockPanel
      title="Filters"
      closeId="browse-filters-close"
      closeLabel="Close filter panel"
      closeTitle={hint('Close', 'toggleFilters')}
      {onClose}
    >
      {#snippet badge()}
        <FilterCount count={activeFilterCount(browseControls.filters)} />
      {/snippet}
      {#snippet actions()}
        {@render saveButton(layout.mode === 'touch')}
        <FilterReset />
      {/snippet}
      {#if !browseView.filterPage}
        {@render saveEntry()}
      {/if}
      <FilterSections {favoriteLocked} />
    </DockPanel>
  {/snippet}
</SaveViewScope>
