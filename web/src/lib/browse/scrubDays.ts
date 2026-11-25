import { dayTitle } from '#lib/browse/dayGroups.ts';
import { monthLabel, type ScrubMonth } from '#lib/browse/scrubberModel.ts';
import type { SortDir } from '#lib/types/search.ts';
import { countOf } from '#lib/utils/plural.ts';

export type ScrubDay = { key: string; count: number };

export type PreviewDay = { key: string; count: number | null };

export type ScrubLabel = { title: string; count: string | null };

export type DayEdges = { keys: string[]; edges: string[] };

const MONTH_KEY = /^(\d{4})-(\d{2})$/;

export function monthDays(month: string): DayEdges | null {
  const match = MONTH_KEY.exec(month);
  if (!match) return null;
  const year = Number(match[1]);
  const index = Number(match[2]) - 1;
  const count = new Date(Date.UTC(year, index + 1, 0)).getUTCDate();
  const edges = Array.from({ length: count + 1 }, (_, day) =>
    new Date(Date.UTC(year, index, day + 1)).toISOString()
  );
  return { keys: edges.slice(0, -1).map((edge) => edge.slice(0, 10)), edges };
}

export function scrubDays(keys: readonly string[], counts: readonly number[]): ScrubDay[] {
  return keys.map((key, at) => ({ key, count: counts[at] ?? 0 })).filter((day) => day.count > 0);
}

export function ordered(days: readonly ScrubDay[], direction: SortDir): ScrubDay[] {
  return direction === 'desc' ? [...days].reverse() : [...days];
}

export function dayAt(days: readonly ScrubDay[], offset: number): ScrubDay | null {
  let seen = 0;
  for (const day of days) {
    seen += day.count;
    if (offset < seen) return day;
  }
  return days.at(-1) ?? null;
}

export function scrubLabel(day: PreviewDay | null, month: ScrubMonth | null): ScrubLabel | null {
  if (day) {
    return {
      title: dayTitle(day.key),
      count: day.count === null ? null : countOf(day.count, 'photo')
    };
  }
  return month ? { title: monthLabel(month.month), count: countOf(month.count, 'photo') } : null;
}
