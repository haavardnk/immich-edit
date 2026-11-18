<script lang="ts">
  import { page } from '$app/state';
  import Disclosure from '#lib/components/Disclosure.svelte';
  import DateFilters from '#lib/components/browse/filters/DateFilters.svelte';
  import ExifFilter from '#lib/components/browse/filters/ExifFilter.svelte';
  import IdPicker from '#lib/components/browse/filters/IdPicker.svelte';
  import FilenameFilter from '#lib/components/browse/filters/FilenameFilter.svelte';
  import LabelFilter from '#lib/components/browse/filters/LabelFilter.svelte';
  import LibraryFilters from '#lib/components/browse/filters/LibraryFilters.svelte';
  import RatingFilter from '#lib/components/browse/filters/RatingFilter.svelte';
  import SizeFilter from '#lib/components/browse/filters/SizeFilter.svelte';
  import { CAMERA_FILTERS, PLACE_FILTERS } from '#lib/browse/exifFilters.ts';
  import { ID_EXCLUSIONS, ID_FILTERS } from '#lib/browse/idFilters.ts';
  import { supportsFilter } from '#lib/browse/browseRoutes.ts';
  import { FILTER_SECTIONS, sectionModified, withoutSection } from '#lib/browse/filterSections.ts';
  import { browseControls, type FilterKey } from '#lib/stores/browseControls.svelte.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';

  let { favoriteLocked = false }: { favoriteLocked?: boolean } = $props();

  const filters = $derived(browseControls.filters);
  const openSections = $derived(new Set(browseView.filterSections));
  const sections = $derived(
    FILTER_SECTIONS.filter((section) => section.keys.some((key) => supports(key)))
  );

  function supports(key: FilterKey): boolean {
    return supportsFilter(page.url, key);
  }
</script>

{#each sections as section (section.id)}
  <Disclosure
    open={openSections.has(section.id)}
    title={section.title}
    modified={sectionModified(filters, section)}
    onReset={() => browseControls.applyFilters(withoutSection(filters, section))}
    onOpenChange={(open) => browseView.setFilterSection(section.id, open)}
  >
    <div class="flex flex-col gap-1 bg-recess px-3 pt-1 pb-2">
      {#if section.id === 'library'}
        <LibraryFilters {favoriteLocked} {supports} />
      {:else if section.id === 'collections'}
        {#each ID_FILTERS.filter(supports) as key (key)}
          <IdPicker {key} excludable={supports(ID_EXCLUSIONS[key].key)} />
        {/each}
      {:else if section.id === 'rating'}
        <RatingFilter />
        <LabelFilter />
      {:else if section.id === 'date'}
        <DateFilters />
      {:else if section.id === 'file'}
        {#if supports('filename')}
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
