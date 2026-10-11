import type { AssetSummary } from './album';
import type { LabelColor } from '$lib/stores/labels';

export type SortDir = 'asc' | 'desc';
export type SortField = 'taken' | 'added' | 'rating';
export type Visibility = 'timeline' | 'archive' | 'hidden';

export type SearchScope = { album: string } | { person: string } | { tag: string };
export type RatingMatch = { eq: number } | { gte: number } | { lte: number } | 'unrated';

export interface BrowseFilter {
  scope?: SearchScope;
  favorite?: boolean;
  visibility?: Visibility;
  imagesOnly?: boolean;
  rating?: RatingMatch;
  filename?: string;
  takenAfter?: string;
  takenBefore?: string;
  rejected?: 'hide' | 'only';
  label?: LabelColor | 'none';
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
