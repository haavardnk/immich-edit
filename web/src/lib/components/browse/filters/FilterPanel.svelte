<script lang="ts">
  import FilterCount from '#lib/components/browse/filters/FilterCount.svelte';
  import FilterSections from '#lib/components/browse/filters/FilterSections.svelte';
  import FilterSheet from '#lib/components/browse/filters/FilterSheet.svelte';
  import FiltersHeading from '#lib/components/browse/filters/FiltersHeading.svelte';
  import ViewBlock from '#lib/components/browse/filters/ViewBlock.svelte';
  import SaveViewScope from '#lib/components/savedViews/SaveViewScope.svelte';
  import { activeFilterCount } from '#lib/browse/filterChips.ts';
  import { VIEW_AND_FILTERS } from '#lib/browse/filterSections.ts';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';
  import { filterPanelHost } from '#lib/commands/grid/filterPanelHost.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { hint } from '#lib/shortcuts/labels.ts';
  import { Button, IconButton } from '@immich/ui';
  import { mdiClose, mdiRestore } from '@mdi/js';
  import { onMount, tick } from 'svelte';

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

  let panel = $state<HTMLElement | null>(null);

  const touch = $derived(layout.mode === 'touch');
  const heading = $derived(touch ? VIEW_AND_FILTERS : 'Filters');
  const active = $derived(activeFilterCount(browseControls.filters));

  async function close(): Promise<void> {
    const hadFocus = panel?.contains(document.activeElement) ?? false;
    browseView.setFiltersOpen(false);
    if (!hadFocus) return;
    await tick();
    document.getElementById('browse-filters-toggle')?.focus();
  }

  onMount(() => filterPanelHost.attach({ close }));
</script>

{#if layout.compact}
  <FilterSheet {favoriteLocked} {count} {sortBasis} {title} />
{:else if browseView.filtersOpen}
  <SaveViewScope {title}>
    {#snippet children(saveButton, saveEntry)}
      <aside
        bind:this={panel}
        id="browse-filters"
        aria-label={heading}
        class="flex w-80 shrink-0 flex-col border-l touch:w-90 border-hairline bg-light max-md:fixed max-md:inset-y-0 max-md:right-0 max-md:z-40 max-md:pt-(--safe-top) max-md:pr-(--safe-right) max-md:pb-(--safe-bottom) max-md:shadow-popover"
      >
        <div
          class="flex h-12 shrink-0 items-center gap-2 border-b border-hairline pr-1.5 pl-3 touch:h-13 touch:pl-4"
        >
          <h2
            class="flex flex-1 items-center gap-1.5 text-[13px] font-semibold whitespace-nowrap text-light-950/90 touch:text-base"
          >
            {heading}
            {#if !touch}
              <FilterCount count={active} />
            {/if}
          </h2>
          {@render saveButton(touch)}
          {#if !touch}
            <Button
              size="tiny"
              variant="ghost"
              color="secondary"
              class="h-8 shrink-0 bg-transparent hover:bg-light-950/6"
              leadingIcon={mdiRestore}
              aria-label="Reset all filters"
              disabled={active === 0}
              onclick={() => browseControls.resetFilters()}
            >
              Reset
            </Button>
          {/if}
          <IconButton
            id="browse-filters-close"
            size="small"
            variant="ghost"
            color="secondary"
            icon={mdiClose}
            title={hint('Close', 'toggleFilters')}
            aria-label="Close filter panel"
            onclick={close}
          />
        </div>
        <div class="min-h-0 flex-1 overflow-y-auto scrollbar-hidden">
          {@render saveEntry()}
          {#if touch}
            <ViewBlock {sortBasis} />
            <FiltersHeading />
          {/if}
          <FilterSections {favoriteLocked} />
        </div>
      </aside>
    {/snippet}
  </SaveViewScope>
{/if}
