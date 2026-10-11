import type { SuggestionField, SuggestionQuery } from '$lib/types/search';

export type ExifFilterKey = SuggestionField;
type SuggestionParent = Exclude<keyof SuggestionQuery, 'field'>;

export const EXIF_GROUPS: ExifFilterKey[][] = [
  ['make', 'model', 'lensModel'],
  ['country', 'state', 'city']
];

export const EXIF_FILTERS: ExifFilterKey[] = EXIF_GROUPS.flat();

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
  const [first, ...rest] = values;
  if (first === undefined) return '';
  return rest.length === 0 ? exifLabel(first) : `${exifLabel(first)} +${rest.length}`;
}

export function exclusiveUnknown(values: string[]): string[] {
  if (values.at(-1) === UNKNOWN_EXIF) return [UNKNOWN_EXIF];
  return values.filter((value) => value !== UNKNOWN_EXIF);
}
