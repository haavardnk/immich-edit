import type { SuggestionField, SuggestionQuery } from '$lib/types/search';
import { listSummary } from './listSummary';

export type ExifFilterKey = SuggestionField;
type SuggestionParent = Exclude<keyof SuggestionQuery, 'field'>;

export const CAMERA_FILTERS: ExifFilterKey[] = ['make', 'model', 'lensModel'];

export const PLACE_FILTERS: ExifFilterKey[] = ['country', 'state', 'city'];

export const EXIF_FILTERS: ExifFilterKey[] = [...CAMERA_FILTERS, ...PLACE_FILTERS];

export const UNKNOWN_EXIF = '';

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
    const known = selected[parent].filter((value) => value !== UNKNOWN_EXIF);
    if (known.length > 0) query[parent] = known;
  }
  return query;
}

export function exifLabel(value: string): string {
  return value === UNKNOWN_EXIF ? 'Unknown' : value;
}

export function exifSummary(values: string[]): string {
  return listSummary(values.map(exifLabel));
}

export function exclusiveUnknown(values: string[]): string[] {
  if (values.at(-1) === UNKNOWN_EXIF) return [UNKNOWN_EXIF];
  return values.filter((value) => value !== UNKNOWN_EXIF);
}
