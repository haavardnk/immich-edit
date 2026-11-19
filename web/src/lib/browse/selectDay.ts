import type { DayGroup } from '#lib/browse/dayGroups.ts';

export type DaySpan = { first: number; last: number; start: number; end: number };

export type DayPager = {
  readonly hasMore: boolean;
  readonly hasPrevious: boolean;
  requestMore: () => Promise<boolean>;
  requestPrevious: () => Promise<boolean>;
};

export type DayPick = {
  pager: DayPager;
  groups: () => readonly DayGroup[];
  ids: (start: number, end: number) => string[];
  isSelected: (id: string) => boolean;
  apply: (ids: string[], on: boolean) => void;
};

export function daySpan(
  groups: readonly DayGroup[],
  fromKey: string,
  toKey: string
): DaySpan | null {
  const from = groups.findIndex((group) => group.key === fromKey);
  const to = groups.findIndex((group) => group.key === toKey);
  if (from < 0 || to < 0) return null;
  const first = Math.min(from, to);
  const last = Math.max(from, to);
  const head = groups[first];
  const tail = groups[last];
  return head && tail ? { first, last, start: head.start, end: tail.end } : null;
}

function loadedCount(groups: readonly DayGroup[]): number {
  return groups.at(-1)?.end ?? 0;
}

export function dayAction(
  ids: readonly string[],
  isSelected: (id: string) => boolean
): 'select' | 'deselect' {
  return ids.length > 0 && ids.every(isSelected) ? 'deselect' : 'select';
}

export async function loadDaySpan(
  pager: DayPager,
  groups: () => readonly DayGroup[],
  fromKey: string,
  toKey: string
): Promise<DaySpan | null> {
  for (;;) {
    const list = groups();
    const span = daySpan(list, fromKey, toKey);
    if (!span) return null;
    const before = span.first === 0 && pager.hasPrevious;
    const after = !before && span.end >= loadedCount(list) && pager.hasMore;
    if (!before && !after) return span;
    const loaded = await (before ? pager.requestPrevious() : pager.requestMore());
    if (!loaded) return null;
    if (loadedCount(groups()) === loadedCount(list)) return daySpan(groups(), fromKey, toKey);
  }
}

export async function pickDays(
  pick: DayPick,
  fromKey: string,
  toKey: string,
  extend: boolean
): Promise<boolean> {
  const span = await loadDaySpan(pick.pager, pick.groups, fromKey, toKey);
  if (!span) return false;
  const ids = pick.ids(span.start, span.end);
  const on = extend || dayAction(ids, pick.isSelected) === 'select';
  pick.apply(ids, on);
  return on;
}
