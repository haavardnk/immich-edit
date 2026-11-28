<script lang="ts">
  import { page } from '$app/state';
  import DateFilter from '#lib/components/browse/filters/DateFilter.svelte';
  import ExifDrill from '#lib/components/browse/filters/ExifDrill.svelte';
  import FilenameFilter from '#lib/components/browse/filters/FilenameFilter.svelte';
  import FilterPage from '#lib/components/browse/filters/FilterPage.svelte';
  import FilterSection from '#lib/components/browse/filters/FilterSection.svelte';
  import IdList from '#lib/components/browse/filters/IdList.svelte';
  import LabelFilter from '#lib/components/browse/filters/LabelFilter.svelte';
  import LibraryFilters from '#lib/components/browse/filters/LibraryFilters.svelte';
  import RatingFilter from '#lib/components/browse/filters/RatingFilter.svelte';
  import SizeFilter from '#lib/components/browse/filters/SizeFilter.svelte';
  import { CAMERA_FILTERS, PLACE_FILTERS } from '#lib/browse/exifFilters.ts';
  import { routeScope, structuredRoute, supportsFilter } from '#lib/browse/browseRoutes.ts';
  import { FILTER_SECTIONS, type FilterSection as Section } from '#lib/browse/filterSections.ts';
  import { filterSummary } from '#lib/browse/filterSummary.ts';
  import { ID_EXCLUSIONS, ID_SCOPES } from '#lib/browse/idFilters.ts';
  import { idName } from '#lib/browse/idOptions.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { browseControls, type FilterKey } from '#lib/stores/browseControls.svelte.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';
  import { tick } from 'svelte';

  let { favoriteLocked = false }: { favoriteLocked?: boolean } = $props();

  const openSections = $derived(new Set(browseView.filterSections));
  const sections = $derived(
    FILTER_SECTIONS.filter((section) => section.keys.some((key) => supports(key)))
  );
  const structured = $derived(structuredRoute(page.url));
  const scope = $derived<Partial<Record<string, string>>>(routeScope(page.url) ?? {});
  const pushes = $derived(layout.mode === 'touch');
  const pageSection = $derived(sections.find((section) => section.id === browseView.filterPage));

  function supports(key: FilterKey): boolean {
    return supportsFilter(page.url, key);
  }

  function showAll(section: Section): (() => void) | undefined {
    return pushes ? () => (browseView.filterPage = section.id) : undefined;
  }

  async function back(id: Section['id']): Promise<void> {
    browseView.filterPage = null;
    await tick();
    const section = document.querySelector<HTMLElement>(`[data-filter-section="${id}"]`);
    if (!section) return;
    section.scrollIntoView({ block: 'start' });
    const target =
      section.querySelector<HTMLElement>('[data-filter-all]') ??
      section.querySelector<HTMLElement>('[data-filter-toggle]');
    target?.focus({ preventScroll: true });
  }
</script>

{#snippet body(section: Section, full: boolean)}
  {#if section.id === 'library'}
    <LibraryFilters {favoriteLocked} {supports} />
  {:else if section.id === 'people' || section.id === 'albums' || section.id === 'tags'}
    <IdList
      key={section.id}
      excludable={supports(ID_EXCLUSIONS[section.id].key)}
      {structured}
      scopeId={scope[ID_SCOPES[section.id]]}
      {full}
      onAll={showAll(section)}
    />
  {:else if section.id === 'rating'}
    <RatingFilter />
    <LabelFilter />
  {:else if section.id === 'date'}
    <DateFilter />
  {:else if section.id === 'camera'}
    <ExifDrill keys={CAMERA_FILTERS} label={section.title} {full} onAll={showAll(section)} />
  {:else if section.id === 'place'}
    <ExifDrill keys={PLACE_FILTERS} label={section.title} {full} onAll={showAll(section)} />
  {:else}
    {#if supports('filename')}
      <FilenameFilter />
    {/if}
    {#if supports('minSize')}
      <SizeFilter />
    {/if}
  {/if}
{/snippet}

{#if pageSection}
  <FilterPage title={pageSection.title} onBack={() => back(pageSection.id)}>
    {@render body(pageSection, true)}
  </FilterPage>
{:else}
  {#each sections as section (section.id)}
    <FilterSection
      id={section.id}
      open={openSections.has(section.id)}
      title={section.title}
      summary={filterSummary(browseControls.filters, section, idName)}
      onOpenChange={(open) => browseView.setFilterSection(section.id, open)}
    >
      {@render body(section, false)}
    </FilterSection>
  {/each}
{/if}
