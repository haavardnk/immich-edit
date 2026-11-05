import type { SortDir, SortField } from '#lib/stores/browseControls.svelte.ts';
import {
  mdiSortAscending,
  mdiSortCalendarAscending,
  mdiSortCalendarDescending,
  mdiSortClockAscending,
  mdiSortClockDescending,
  mdiSortDescending,
  mdiSortNumericAscending,
  mdiSortNumericDescending
} from '@mdi/js';

export const SORT_ICONS: Readonly<Record<SortField, Readonly<Record<SortDir, string>>>> = {
  taken: { asc: mdiSortCalendarAscending, desc: mdiSortCalendarDescending },
  added: { asc: mdiSortClockAscending, desc: mdiSortClockDescending },
  rating: { asc: mdiSortNumericAscending, desc: mdiSortNumericDescending },
  size: { asc: mdiSortAscending, desc: mdiSortDescending }
};

export const SORT_FIELD_LABELS: Readonly<Record<SortField, string>> = {
  taken: 'Date taken',
  added: 'Date added',
  rating: 'Rating',
  size: 'Size'
};

export const SORT_DIRECTION_LABELS: Readonly<Record<SortField, Readonly<Record<SortDir, string>>>> =
  {
    taken: { desc: 'Newest first', asc: 'Oldest first' },
    added: { desc: 'Newest first', asc: 'Oldest first' },
    rating: { desc: 'Highest first', asc: 'Lowest first' },
    size: { desc: 'Largest first', asc: 'Smallest first' }
  };

export const EDIT_SORT_LABELS: Readonly<Record<SortDir, string>> = {
  desc: 'Newest edit first',
  asc: 'Oldest edit first'
};

export const EDIT_SORT_ICONS: Readonly<Record<SortDir, string>> = {
  asc: mdiSortAscending,
  desc: mdiSortDescending
};

const STRUCTURED_FIELDS: ReadonlySet<SortField> = new Set(['rating', 'size']);

export function sortFields(structuredSearch: boolean): SortField[] {
  return (Object.keys(SORT_FIELD_LABELS) as SortField[]).filter(
    (field) => structuredSearch || !STRUCTURED_FIELDS.has(field)
  );
}

export function sortLabel(field: SortField, dir: SortDir): string {
  return `Sort: ${SORT_FIELD_LABELS[field].toLowerCase()}, ${SORT_DIRECTION_LABELS[field][dir].toLowerCase()}`;
}
