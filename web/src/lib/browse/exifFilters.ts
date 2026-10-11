import type { SuggestionField, SuggestionQuery } from '$lib/types/search';

export type ExifFilterKey = SuggestionField;
type SuggestionParent = Exclude<keyof SuggestionQuery, 'field'>;

export const CAMERA_FILTERS: ExifFilterKey[] = ['make', 'model', 'lensModel'];

export const PLACE_FILTERS: ExifFilterKey[] = ['country', 'state', 'city'];

export const EXIF_FILTERS: ExifFilterKey[] = [...CAMERA_FILTERS, ...PLACE_FILTERS];

export const EXIF_LABELS: Record<ExifFilterKey, string> = {
  make: 'Make',
  model: 'Model',
  lensModel: 'Lens',
  country: 'Country',
  state: 'State',
  city: 'City'
};

const PARENTS: Record<ExifFilterKey, SuggestionParent[]> = {
  make: [],
  model: ['make'],
  lensModel: ['make', 'model'],
  country: [],
  state: ['country'],
  city: ['country', 'state']
};

export function suggestionQuery(
  key: ExifFilterKey,
  selected: Record<ExifFilterKey, string[]>
): SuggestionQuery {
  const query: SuggestionQuery = { field: key };
  for (const parent of PARENTS[key]) {
    if (selected[parent].length > 0) query[parent] = [...selected[parent]];
  }
  return query;
}

export function exifSummary(values: string[]): string {
  if (values.length <= 1) return values[0] ?? '';
  return `${values[0]} +${values.length - 1}`;
}
