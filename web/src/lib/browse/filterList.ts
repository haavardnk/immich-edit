import { filterByName } from '#lib/browse/listFilter.ts';

export interface FilterListItem {
  id: string;
  label: string;
  count?: number | null;
}

export interface FilterRows<T extends FilterListItem> {
  pinned: T[];
  rest: T[];
  hidden: boolean;
}

export const PREVIEW_ROWS = 5;

function byLabel<T extends FilterListItem>(a: T, b: T): number {
  return a.label.localeCompare(b.label);
}

function byCount<T extends FilterListItem>(a: T, b: T): number {
  return (b.count ?? 0) - (a.count ?? 0) || byLabel(a, b);
}

export function filterRows<T extends FilterListItem>(
  items: readonly T[],
  isSelected: (id: string) => boolean,
  query: string,
  full: boolean
): FilterRows<T> {
  const pinned = items.filter((item) => isSelected(item.id)).sort(byLabel);
  const others = items.filter((item) => !isSelected(item.id));
  if (query) {
    return {
      pinned,
      rest: filterByName(others, query, (item) => item.label).sort(byLabel),
      hidden: false
    };
  }
  if (full) return { pinned, rest: [...others].sort(byLabel), hidden: false };
  const rest = [...others].sort(byCount).slice(0, PREVIEW_ROWS);
  return { pinned, rest, hidden: rest.length < others.length };
}
