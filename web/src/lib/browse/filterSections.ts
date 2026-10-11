import { FILTER_DEFAULTS, type BrowseFilters } from '$lib/stores/browseControls.svelte';
import { CAMERA_FILTERS, PLACE_FILTERS } from '$lib/browse/exifFilters';
import type { FilterKey } from '$lib/browse/filterChips';

export type FilterSectionId = 'library' | 'rating' | 'date' | 'camera' | 'place' | 'file';

export interface FilterSection {
  id: FilterSectionId;
  title: string;
  keys: FilterKey[];
}

export const FILTER_SECTIONS: FilterSection[] = [
  { id: 'library', title: 'Library', keys: ['visibility', 'rejected', 'favoriteOnly'] },
  { id: 'rating', title: 'Rating & label', keys: ['rating', 'label'] },
  { id: 'date', title: 'Date', keys: ['takenAfter', 'takenBefore'] },
  { id: 'camera', title: 'Camera', keys: CAMERA_FILTERS },
  { id: 'place', title: 'Place', keys: PLACE_FILTERS },
  { id: 'file', title: 'File', keys: ['filename'] }
];

export function isFilterSectionId(value: unknown): value is FilterSectionId {
  return FILTER_SECTIONS.some((section) => section.id === value);
}

export function sectionModified(filters: BrowseFilters, section: FilterSection): boolean {
  return section.keys.some((key) => {
    const value = filters[key];
    return Array.isArray(value) ? value.length > 0 : value !== FILTER_DEFAULTS[key];
  });
}

export function withoutSection(filters: BrowseFilters, section: FilterSection): BrowseFilters {
  return {
    ...filters,
    ...Object.fromEntries(section.keys.map((key) => [key, FILTER_DEFAULTS[key]]))
  };
}
