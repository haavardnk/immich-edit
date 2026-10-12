import {
  FILTER_DEFAULTS,
  isDefaultFilter,
  type BrowseFilters,
  type FilterKey
} from '$lib/stores/browseControls.svelte';
import { CAMERA_FILTERS, PLACE_FILTERS } from '$lib/browse/exifFilters';
import { ID_EXCLUSIONS, ID_FILTERS } from '$lib/browse/idFilters';

export type FilterSectionId =
  'library' | 'collections' | 'rating' | 'date' | 'camera' | 'place' | 'file';

export interface FilterSection {
  id: FilterSectionId;
  title: string;
  keys: FilterKey[];
}

export const FILTER_SECTIONS: FilterSection[] = [
  {
    id: 'library',
    title: 'Library',
    keys: ['visibility', 'rejected', 'favoriteOnly']
  },
  {
    id: 'collections',
    title: 'Albums, people & tags',
    keys: ID_FILTERS.flatMap((key) => [key, ID_EXCLUSIONS[key].key])
  },
  { id: 'rating', title: 'Rating & label', keys: ['rating', 'label'] },
  { id: 'date', title: 'Date', keys: ['takenAfter', 'takenBefore', 'addedAfter', 'addedBefore'] },
  { id: 'camera', title: 'Camera', keys: CAMERA_FILTERS },
  { id: 'place', title: 'Place', keys: PLACE_FILTERS },
  { id: 'file', title: 'File', keys: ['filename', 'minSize', 'maxSize'] }
];

export function isFilterSectionId(value: unknown): value is FilterSectionId {
  return FILTER_SECTIONS.some((section) => section.id === value);
}

export function sectionModified(filters: BrowseFilters, section: FilterSection): boolean {
  return section.keys.some((key) => !isDefaultFilter(filters, key));
}

export function withoutSection(filters: BrowseFilters, section: FilterSection): BrowseFilters {
  return {
    ...filters,
    ...Object.fromEntries(section.keys.map((key) => [key, FILTER_DEFAULTS[key]]))
  };
}
