<script lang="ts">
  import FilterSections from '#lib/components/browse/filters/FilterSections.svelte';
  import FilterSheet from '#lib/components/browse/filters/FilterSheet.svelte';
  import { browseControls } from '#lib/stores/browseControls.svelte.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';
  import { filterPanelHost } from '#lib/commands/grid/filterPanelHost.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { hint } from '#lib/shortcuts/labels.ts';
  import { Button, IconButton } from '@immich/ui';
  import { mdiClose, mdiRestore } from '@mdi/js';
  import { onMount, tick } from 'svelte';

  let { favoriteLocked = false, count }: { favoriteLocked?: boolean; count?: number } = $props();

  let panel = $state<HTMLElement | null>(null);

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
  <FilterSheet {favoriteLocked} {count} />
{:else if browseView.filtersOpen}
  <aside
    bind:this={panel}
    id="browse-filters"
    aria-label="Filters"
    class="flex w-80 shrink-0 flex-col border-l border-hairline bg-light max-md:fixed max-md:inset-y-0 max-md:right-0 max-md:z-40 max-md:pt-(--safe-top) max-md:pr-(--safe-right) max-md:pb-(--safe-bottom) max-md:shadow-popover"
  >
    <div class="flex h-12 shrink-0 items-center gap-2 border-b border-hairline pr-1.5 pl-3">
      <h2 class="min-w-0 flex-1 truncate text-[13px] font-semibold text-light-950/90">Filters</h2>
      <Button
        size="tiny"
        variant="ghost"
        color="secondary"
        class="h-8 bg-transparent hover:bg-light-950/6"
        leadingIcon={mdiRestore}
        aria-label="Reset all filters"
        disabled={!browseControls.isFiltered}
        onclick={() => browseControls.resetFilters()}
      >
        Reset
      </Button>
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
      <FilterSections {favoriteLocked} />
    </div>
  </aside>
{/if}
