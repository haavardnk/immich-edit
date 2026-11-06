import {
  FILTER_DEFAULTS,
  type BrowseFilters,
  type FilterKey
} from '#lib/stores/browseControls.svelte.ts';
import { filterChip, filterValueLabel, type FilterNameOf } from '#lib/browse/filterChips.ts';
import type { FilterSection, FilterSectionId } from '#lib/browse/filterSections.ts';

const IDLE: Record<FilterSectionId, FilterKey[] | string> = {
  library: ['visibility', 'rejected'],
  collections: 'Any',
  rating: ['rating', 'label'],
  date: 'Any date',
  camera: 'Any camera',
  place: 'Any place',
  file: 'Any name or size'
};

const SEPARATOR = ' · ';

function lowerFirst(text: string): string {
  return text.charAt(0).toLowerCase() + text.slice(1);
}

function idleSummary(
  section: FilterSection,
  nameOf: FilterNameOf,
  supports: (key: FilterKey) => boolean
): string {
  const idle = IDLE[section.id];
  if (typeof idle === 'string') return idle;
  const [first, ...rest] = idle
    .filter(supports)
    .map((key) => filterValueLabel(FILTER_DEFAULTS, key, nameOf));
  return [first ?? 'Any', ...rest.map(lowerFirst)].join(SEPARATOR);
}

export function filterSummary(
  filters: BrowseFilters,
  section: FilterSection,
  nameOf: FilterNameOf,
  supports: (key: FilterKey) => boolean = () => true
): string {
  const chips = section.keys.flatMap((key) => filterChip(filters, key, nameOf) ?? []);
  if (chips.length === 0) return idleSummary(section, nameOf, supports);
  return chips.map((chip) => chip.label).join(SEPARATOR);
}
