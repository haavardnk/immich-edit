<script lang="ts">
  import { page } from '$app/state';
  import CollectionFilters from '#lib/components/browse/filters/CollectionFilters.svelte';
  import DateFilter from '#lib/components/browse/filters/DateFilter.svelte';
  import ExifDrill from '#lib/components/browse/filters/ExifDrill.svelte';
  import FilenameFilter from '#lib/components/browse/filters/FilenameFilter.svelte';
  import FilterSection from '#lib/components/browse/filters/FilterSection.svelte';
  import LabelFilter from '#lib/components/browse/filters/LabelFilter.svelte';
  import LibraryFilters from '#lib/components/browse/filters/LibraryFilters.svelte';
  import RatingFilter from '#lib/components/browse/filters/RatingFilter.svelte';
  import SizeFilter from '#lib/components/browse/filters/SizeFilter.svelte';
  import { CAMERA_FILTERS, PLACE_FILTERS } from '#lib/browse/exifFilters.ts';
  import { supportsFilter } from '#lib/browse/browseRoutes.ts';
  import { FILTER_SECTIONS } from '#lib/browse/filterSections.ts';
  import { filterSummary } from '#lib/browse/filterSummary.ts';
  import { idName } from '#lib/browse/idOptions.ts';
  import { browseControls, type FilterKey } from '#lib/stores/browseControls.svelte.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';

  let { favoriteLocked = false }: { favoriteLocked?: boolean } = $props();

  const openSections = $derived(new Set(browseView.filterSections));
  const sections = $derived(
    FILTER_SECTIONS.filter((section) => section.keys.some((key) => supports(key)))
  );

  function supports(key: FilterKey): boolean {
    return supportsFilter(page.url, key);
  }
</script>

{#each sections as section (section.id)}
  <FilterSection
    open={openSections.has(section.id)}
    title={section.title}
    summary={filterSummary(browseControls.filters, section, idName)}
    onOpenChange={(open) => browseView.setFilterSection(section.id, open)}
  >
    {#if section.id === 'library'}
      <LibraryFilters {favoriteLocked} {supports} />
    {:else if section.id === 'collections'}
      <CollectionFilters {supports} />
    {:else if section.id === 'rating'}
      <RatingFilter />
      <LabelFilter />
    {:else if section.id === 'date'}
      <DateFilter />
    {:else if section.id === 'camera'}
      <ExifDrill keys={CAMERA_FILTERS} label={section.title} />
    {:else if section.id === 'place'}
      <ExifDrill keys={PLACE_FILTERS} label={section.title} />
    {:else}
      {#if supports('filename')}
        <FilenameFilter />
      {/if}
      {#if supports('minSize')}
        <SizeFilter />
      {/if}
    {/if}
  </FilterSection>
{/each}
