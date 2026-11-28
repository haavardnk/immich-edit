import type { DayGroup } from '#lib/browse/dayGroups.ts';

export type DaySpan = { first: number; last: number; start: number; end: number };

export type DayPager = {
  readonly hasMore: boolean;
  readonly hasPrevious: boolean;
  requestMore: () => Promise<boolean>;
  requestPrevious: () => Promise<boolean>;
};

export type DayFeed = {
  pager: DayPager;
  groups: () => readonly DayGroup[];
  indexOf: (id: string) => number;
};

export type DayPick = DayFeed & {
  ids: (start: number, end: number) => string[];
  isSelected: (id: string) => boolean;
  apply: (ids: string[], on: boolean) => void;
};

function dayOf(groups: readonly DayGroup[], index: number): number {
  if (index < 0) return -1;
  return groups.findIndex((group) => group.start <= index && index < group.end);
}

export function daySpan(
  groups: readonly DayGroup[],
  indexOf: (id: string) => number,
  fromId: string,
  toId: string
): DaySpan | null {
  const from = dayOf(groups, indexOf(fromId));
  const to = dayOf(groups, indexOf(toId));
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
  { pager, groups, indexOf }: DayFeed,
  fromId: string,
  toId: string
): Promise<DaySpan | null> {
  for (;;) {
    const list = groups();
    const span = daySpan(list, indexOf, fromId, toId);
    if (!span) return null;
    const before = span.first === 0 && pager.hasPrevious;
    const after = !before && span.end >= loadedCount(list) && pager.hasMore;
    if (!before && !after) return span;
    const loaded = await (before ? pager.requestPrevious() : pager.requestMore());
    if (!loaded) return null;
    if (loadedCount(groups()) === loadedCount(list))
      return daySpan(groups(), indexOf, fromId, toId);
  }
}

export async function pickDays(pick: DayPick, fromId: string, toId: string): Promise<boolean> {
  const span = await loadDaySpan(pick, fromId, toId);
  if (!span) return false;
  const ids = pick.ids(span.start, span.end);
  const on = span.first !== span.last || dayAction(ids, pick.isSelected) === 'select';
  pick.apply(ids, on);
  return on;
}
