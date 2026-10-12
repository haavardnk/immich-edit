import type { IdSet } from '$lib/types/search';
import { listSummary } from './listSummary';

export type IdMatch = 'all' | 'any' | 'none';
export type IdFilterKey = 'albums' | 'people' | 'tags';

export interface IdFilter {
  ids: string[];
  match: IdMatch;
}

export const ID_FILTERS: IdFilterKey[] = ['albums', 'people', 'tags'];

export const ID_MATCHES: IdMatch[] = ['all', 'any', 'none'];

export const ID_MATCH_LABELS: Record<IdMatch, string> = {
  all: 'All of',
  any: 'Any of',
  none: 'None of'
};

export const ID_LABELS: Record<IdFilterKey, string> = {
  albums: 'Albums',
  people: 'People',
  tags: 'Tags'
};

export const ID_SCOPES: Record<IdFilterKey, 'album' | 'person' | 'tag'> = {
  albums: 'album',
  people: 'person',
  tags: 'tag'
};

export const ID_EXCLUSIONS: Record<
  IdFilterKey,
  { key: 'notInAlbum' | 'noPeople' | 'untagged'; label: string }
> = {
  albums: { key: 'notInAlbum', label: 'Not in any album' },
  people: { key: 'noPeople', label: 'No people' },
  tags: { key: 'untagged', label: 'Untagged' }
};

export const NO_IDS: IdFilter = { ids: [], match: 'all' };

export function idFilter(ids: string[], match: IdMatch): IdFilter {
  return ids.length > 0 ? { ids, match } : NO_IDS;
}

export function idSet({ ids, match }: IdFilter, structured: boolean): IdSet | undefined {
  if (ids.length === 0) return undefined;
  if (structured) return { [match]: ids };
  if (match === 'all' || (match === 'any' && ids.length === 1)) return { all: ids };
  return undefined;
}

export function idSummary(ids: string[], nameOf: (id: string) => string): string {
  return listSummary(ids.map(nameOf));
}

export function idChipLabel(
  key: IdFilterKey,
  { ids, match }: IdFilter,
  nameOf: (id: string) => string
): string {
  const names = idSummary(ids, nameOf);
  const label = ID_LABELS[key];
  if (match === 'any') return `Any of ${label.toLowerCase()}: ${names}`;
  if (match === 'none') return `Without ${label.toLowerCase()}: ${names}`;
  return `${label}: ${names}`;
}
