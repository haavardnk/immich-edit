import type { AssetSummary } from './album';
import type { LabelColor } from '$lib/stores/labels';

export type SortDir = 'asc' | 'desc';
export type SortField = 'taken' | 'added' | 'rating' | 'size';
export type Visibility = 'timeline' | 'archive' | 'hidden';

export type SearchScope = { album: string } | { person: string } | { tag: string };
export type RatingMatch = { eq: number } | { gte: number } | { lte: number } | 'unrated';
export type IdSet = { all?: string[]; any?: string[]; none?: string[] };

export interface BrowseFilter {
  scope?: SearchScope;
  favorite?: boolean;
  notInAlbum?: boolean;
  untagged?: boolean;
  noPeople?: boolean;
  albums?: IdSet;
  people?: IdSet;
  tags?: IdSet;
  visibility?: Visibility;
  imagesOnly?: boolean;
  rating?: RatingMatch;
  filename?: string;
  minSize?: number;
  maxSize?: number;
  takenAfter?: string;
  takenBefore?: string;
  addedAfter?: string;
  addedBefore?: string;
  rejected?: 'hide' | 'only';
  label?: LabelColor | 'none';
  make?: (string | null)[];
  model?: (string | null)[];
  lensModel?: (string | null)[];
  country?: (string | null)[];
  state?: (string | null)[];
  city?: (string | null)[];
}

export type SuggestionField = 'make' | 'model' | 'lensModel' | 'country' | 'state' | 'city';

export interface SuggestionQuery {
  field: SuggestionField;
  make?: string[];
  model?: string[];
  country?: string[];
  state?: string[];
}

export interface SearchSort {
  field: SortField;
  direction: SortDir;
}

export interface PageQuery {
  filter: BrowseFilter;
  sort?: SearchSort;
  size: number;
  cursor?: string;
  withExif?: boolean;
}

export interface SmartQuery {
  text: string;
  filter: BrowseFilter;
  size: number;
  cursor?: string;
  withExif?: boolean;
}

export interface SearchPage {
  items: AssetSummary[];
  nextCursor: string | null;
}
