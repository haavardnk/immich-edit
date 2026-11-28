import type { FilterKey } from '#lib/stores/browseControls.svelte.ts';
import { CAMERA_FILTERS, PLACE_FILTERS } from '#lib/browse/exifFilters.ts';
import { ID_EXCLUSIONS, ID_LABELS, type IdFilterKey } from '#lib/browse/idFilters.ts';

const ID_LIST_ORDER: readonly IdFilterKey[] = ['people', 'albums', 'tags'];

export type FilterSectionId =
  'library' | 'people' | 'albums' | 'tags' | 'rating' | 'date' | 'camera' | 'place' | 'file';

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
  { id: 'rating', title: 'Rating & label', keys: ['rating', 'label'] },
  ...ID_LIST_ORDER.map((key) => ({
    id: key,
    title: ID_LABELS[key],
    keys: [key, ID_EXCLUSIONS[key].key] satisfies FilterKey[]
  })),
  { id: 'date', title: 'Date', keys: ['takenAfter', 'takenBefore', 'addedAfter', 'addedBefore'] },
  { id: 'camera', title: 'Camera', keys: CAMERA_FILTERS },
  { id: 'place', title: 'Place', keys: PLACE_FILTERS },
  { id: 'file', title: 'File', keys: ['filename', 'minSize', 'maxSize'] }
];

export function isFilterSectionId(value: unknown): value is FilterSectionId {
  return FILTER_SECTIONS.some((section) => section.id === value);
}
