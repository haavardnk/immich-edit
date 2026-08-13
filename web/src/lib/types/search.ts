import type { AssetSummary } from './album';

export type SortDir = 'asc' | 'desc';
export type Visibility = 'timeline' | 'archive' | 'hidden';

export interface SearchQuery {
  query?: string;
  albumIds?: string[];
  personIds?: string[];
  tagIds?: string[];
  originalFileName?: string;
  isFavorite?: boolean;
  rating?: number | null;
  visibility?: Visibility;
  order?: SortDir;
  type?: 'IMAGE';
  withExif?: boolean;
  size?: number;
  page?: string;
  takenAfter?: string;
  takenBefore?: string;
}

export interface SearchResult {
  items: AssetSummary[];
  count: number;
  total: number;
  nextPage: string | null;
}
