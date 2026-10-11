<script lang="ts">
  import { page } from '$app/state';
  import Disclosure from '$lib/components/Disclosure.svelte';
  import DateFilters from '$lib/components/browse/filters/DateFilters.svelte';
  import ExifFilter from '$lib/components/browse/filters/ExifFilter.svelte';
  import FilenameFilter from '$lib/components/browse/filters/FilenameFilter.svelte';
  import LabelFilter from '$lib/components/browse/filters/LabelFilter.svelte';
  import LibraryFilters from '$lib/components/browse/filters/LibraryFilters.svelte';
  import RatingFilter from '$lib/components/browse/filters/RatingFilter.svelte';
  import SizeFilter from '$lib/components/browse/filters/SizeFilter.svelte';
  import { CAMERA_FILTERS, PLACE_FILTERS } from '$lib/browse/exifFilters';
  import { supportsFilter } from '$lib/browse/browseRoutes';
  import { FILTER_SECTIONS, sectionModified, withoutSection } from '$lib/browse/filterSections';
  import type { FilterKey } from '$lib/browse/filterChips';
  import { browseControls, FILTER_DEFAULTS } from '$lib/stores/browseControls.svelte';
  import { browseView } from '$lib/stores/browseView.svelte';
  import { hint } from '$lib/shortcuts/labels';
  import { matchKeybind } from '$lib/shortcuts/match';
  import { isTypingTarget } from '$lib/shortcuts/targets';
  import { Button, IconButton } from '@immich/ui';
  import { mdiClose, mdiRestore } from '@mdi/js';
  import { tick } from 'svelte';

  let {
    favoriteLocked = false,
    hideFilenameFilter = false
  }: { favoriteLocked?: boolean; hideFilenameFilter?: boolean } = $props();

  const filters = $derived(browseControls.filters);
  const openSections = $derived(new Set(browseView.filterSections));
  const sections = $derived(
    FILTER_SECTIONS.filter(
      (section) => section.id !== 'file' || !hideFilenameFilter || supports('minSize')
    )
  );

  let panel = $state<HTMLElement | null>(null);

  function supports(key: FilterKey): boolean {
    return supportsFilter(page.url, key);
  }

  async function close(): Promise<void> {
    const hadFocus = panel?.contains(document.activeElement) ?? false;
    browseView.setFiltersOpen(false);
    if (!hadFocus) return;
    await tick();
    document.getElementById('browse-filters-toggle')?.focus();
  }

  function onKeydown(e: KeyboardEvent): void {
    if (browseView.loupeId || isTypingTarget(e)) return;
    if (matchKeybind(e, ['grid']) !== 'toggleFilters') return;
    e.preventDefault();
    if (browseView.filtersOpen) void close();
    else browseView.setFiltersOpen(true);
  }
</script>

<svelte:window onkeydown={onKeydown} />

{#if browseView.filtersOpen}
  <aside
    bind:this={panel}
    id="browse-filters"
    aria-label="Filters"
    class="flex w-80 shrink-0 flex-col border-l border-hairline bg-light max-md:fixed max-md:inset-y-0 max-md:right-0 max-md:z-40 max-md:shadow-popover"
  >
    <div class="flex h-12 shrink-0 items-center gap-2 border-b border-hairline pr-1.5 pl-3">
      <h2 class="min-w-0 flex-1 truncate text-[13px] font-semibold text-white/90">Filters</h2>
      <Button
        size="tiny"
        variant="ghost"
        color="secondary"
        class="h-8 bg-transparent hover:bg-white/6"
        leadingIcon={mdiRestore}
        aria-label="Reset all filters"
        disabled={!browseControls.isFiltered}
        onclick={() => browseControls.applyFilters(FILTER_DEFAULTS)}
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
      {#each sections as section (section.id)}
        <Disclosure
          open={openSections.has(section.id)}
          title={section.title}
          modified={sectionModified(filters, section)}
          onReset={() => browseControls.applyFilters(withoutSection(filters, section))}
          onOpenChange={(open) => browseView.setFilterSection(section.id, open)}
        >
          <div class="flex flex-col gap-1 bg-black/10 px-3 pt-1 pb-2">
            {#if section.id === 'library'}
              <LibraryFilters {favoriteLocked} {supports} />
            {:else if section.id === 'rating'}
              <RatingFilter />
              <LabelFilter />
            {:else if section.id === 'date'}
              <DateFilters />
            {:else if section.id === 'file'}
              {#if !hideFilenameFilter}
                <FilenameFilter />
              {/if}
              {#if supports('minSize')}
                <SizeFilter key="minSize" label="Min size (MB)" />
                <SizeFilter key="maxSize" label="Max size (MB)" />
              {/if}
            {:else}
              {#each section.id === 'camera' ? CAMERA_FILTERS : PLACE_FILTERS as key (key)}
                <ExifFilter {key} />
              {/each}
            {/if}
          </div>
        </Disclosure>
      {/each}
    </div>
  </aside>
{/if}
