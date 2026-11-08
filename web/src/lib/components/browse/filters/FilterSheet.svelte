<script lang="ts">
  import FilterSections from '#lib/components/browse/filters/FilterSections.svelte';
  import ViewSection from '#lib/components/browse/filters/ViewSection.svelte';
  import SaveViewScope from '#lib/components/savedViews/SaveViewScope.svelte';
  import BottomSheet from '#lib/components/sheet/BottomSheet.svelte';
  import { presence } from '#lib/components/sheet/presence.svelte.ts';
  import { showPhotosLabel } from '#lib/browse/filterChips.ts';
  import { VIEW_AND_FILTERS } from '#lib/browse/filterSections.ts';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';
  import { Button } from '@immich/ui';

  let {
    favoriteLocked = false,
    count,
    sortBasis,
    title
  }: {
    favoriteLocked?: boolean;
    count?: number;
    sortBasis: 'capture' | 'edit' | null;
    title: string;
  } = $props();

  const sheet = presence(() => browseView.filtersOpen);
</script>

{#if sheet.mounted}
  <SaveViewScope {title}>
    {#snippet children(saveButton, saveEntry)}
      <BottomSheet
        open={browseView.filtersOpen}
        onOpenChange={(open) => browseView.setFiltersOpen(open)}
        onExited={sheet.exited}
        title={VIEW_AND_FILTERS}
        close="icon"
        closeLabel="Close {VIEW_AND_FILTERS.toLowerCase()}"
        detents={[{ fraction: 1 }]}
      >
        {#snippet header()}
          {@render saveButton(true)}
        {/snippet}
        {@render saveEntry()}
        <ViewSection {sortBasis} />
        <FilterSections {favoriteLocked} />
        {#snippet footer()}
          <div class="flex w-full gap-2.5">
            <Button
              size="medium"
              variant="ghost"
              color="secondary"
              shape="round"
              class="h-12 w-28 shrink-0 bg-light-200 text-base font-semibold"
              aria-label="Reset all filters"
              disabled={!browseControls.isFiltered}
              onclick={() => browseControls.resetFilters()}
            >
              Reset
            </Button>
            <Button
              size="medium"
              color="primary"
              shape="round"
              class="h-12 flex-1 text-base font-semibold"
              onclick={() => browseView.setFiltersOpen(false)}
            >
              {showPhotosLabel(count)}
            </Button>
          </div>
        {/snippet}
      </BottomSheet>
    {/snippet}
  </SaveViewScope>
{/if}
