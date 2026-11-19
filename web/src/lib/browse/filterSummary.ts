import type { BrowseFilters } from '#lib/stores/browseControls.svelte.ts';
import { filterValue, type FilterNameOf } from '#lib/browse/filterChips.ts';
import type { FilterSection } from '#lib/browse/filterSections.ts';

export function filterSummary(
  filters: BrowseFilters,
  section: FilterSection,
  nameOf: FilterNameOf
): string | undefined {
  const labels = section.keys.flatMap((key) => filterValue(filters, key, nameOf) ?? []);
  return labels.length > 0 ? labels.join(' · ') : undefined;
}
