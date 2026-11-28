<script lang="ts">
  import { reveal } from '#lib/motion/reveal.ts';
  import DockPanel from '#lib/components/browse/DockPanel.svelte';
  import FilterDock from '#lib/components/browse/filters/FilterDock.svelte';
  import FilterSheet from '#lib/components/browse/filters/FilterSheet.svelte';
  import ViewBody from '#lib/components/browse/view/ViewBody.svelte';
  import ViewSheet from '#lib/components/browse/view/ViewSheet.svelte';
  import { browseView } from '#lib/stores/browseView.svelte.ts';
  import { filterPanelHost } from '#lib/commands/grid/filterPanelHost.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
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

  const dock = $derived(browseView.filtersOpen ? 'filters' : browseView.viewOpen ? 'view' : null);

  async function close(): Promise<void> {
    const toggle = dock === 'view' ? 'browse-view-toggle' : 'browse-filters-toggle';
    const hadFocus = panel?.contains(document.activeElement) ?? false;
    if (dock === 'view') browseView.setViewOpen(false);
    else browseView.setFiltersOpen(false);
    if (!hadFocus) return;
    await tick();
    document.getElementById(toggle)?.focus();
  }

  onMount(() => filterPanelHost.attach({ close }));
</script>

{#if layout.compact}
  <FilterSheet {favoriteLocked} {count} {title} />
  <ViewSheet {sortBasis} />
{:else if dock}
  <aside
    bind:this={panel}
    id={dock === 'view' ? 'browse-view' : 'browse-filters'}
    aria-label={dock === 'view' ? 'View' : 'Filters'}
    transition:reveal={{ axis: 'x' }}
    class="flex w-80 shrink-0 flex-col border-l touch:w-90 border-hairline bg-light max-md:fixed max-md:inset-y-0 max-md:right-0 max-md:z-40 max-md:pt-(--safe-top) max-md:pr-(--safe-right) max-md:pb-(--safe-bottom) max-md:shadow-popover"
  >
    {#if dock === 'view'}
      <DockPanel
        title="View"
        closeId="browse-view-close"
        closeLabel="Close view panel"
        closeTitle="Close"
        onClose={() => void close()}
      >
        <ViewBody {sortBasis} />
      </DockPanel>
    {:else}
      <FilterDock {favoriteLocked} {title} onClose={() => void close()} />
    {/if}
  </aside>
{/if}
