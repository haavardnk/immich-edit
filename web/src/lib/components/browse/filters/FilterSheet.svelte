<script lang="ts">
  import FilterReset from '#lib/components/browse/filters/FilterReset.svelte';
  import FilterSections from '#lib/components/browse/filters/FilterSections.svelte';
  import SaveViewScope from '#lib/components/savedViews/SaveViewScope.svelte';
  import BottomSheet from '#lib/components/sheet/BottomSheet.svelte';
  import SheetActionButton from '#lib/components/sheet/SheetActionButton.svelte';
  import { presence } from '#lib/components/sheet/presence.svelte.ts';
  import { activeFilterCount, showPhotosLabel } from '#lib/browse/filterChips.ts';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';

  let {
    favoriteLocked = false,
    count,
    title
  }: {
    favoriteLocked?: boolean;
    count?: number;
    title: string;
  } = $props();

  const sheet = presence(() => browseView.filtersOpen);
  const active = $derived(activeFilterCount(browseControls.filters));
</script>

{#if sheet.mounted}
  <SaveViewScope {title}>
    {#snippet children(saveButton, saveEntry)}
      <BottomSheet
        open={browseView.filtersOpen}
        onOpenChange={(open) => browseView.setFiltersOpen(open)}
        onExited={sheet.exited}
        title="Filters"
        subtitle={active > 0 ? `${active} active` : undefined}
        close="icon"
        closeLabel="Close filters"
        detents={[{ fraction: 1 }]}
      >
        {#snippet header()}
          <FilterReset />
          {@render saveButton(true)}
        {/snippet}
        {#if !browseView.filterPage}
          {@render saveEntry()}
        {/if}
        <FilterSections {favoriteLocked} />
        {#snippet footer()}
          <SheetActionButton onclick={() => browseView.setFiltersOpen(false)}>
            {showPhotosLabel(count)}
          </SheetActionButton>
        {/snippet}
      </BottomSheet>
    {/snippet}
  </SaveViewScope>
{/if}
