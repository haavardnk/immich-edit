import type { MonthCount, SortDir } from '#lib/types/search.ts';

export type ScrubMonth = {
  month: string;
  year: number;
  count: number;
  start: number;
  end: number;
};

export type ScrubModel = { months: ScrubMonth[]; total: number };

export type ScrubPoint = { index: number; within: number };

export type ScrubMark = { label: string; at: number };

export type LoadedPhoto = { id: string; exact: boolean };

export type MonthAnchor = { month: string; offset: number; id: string };

export type CaptureClock = 'photo' | 'utc';

type Dated = {
  id: string;
  fileCreatedAt: string | null;
  localDateTime?: string | null;
  copyOf?: string;
};

const MONTH_KEY = /^(\d{4})-(\d{2})$/;

function capturedAt(item: Dated | undefined, clock: CaptureClock): Date | null {
  const raw = clock === 'photo' ? item?.localDateTime : item?.fileCreatedAt;
  if (!raw) return null;
  const at = new Date(raw);
  return Number.isNaN(at.getTime()) ? null : at;
}

export function captureDay(item: Dated | undefined, clock: CaptureClock): string | null {
  return capturedAt(item, clock)?.toISOString().slice(0, 10) ?? null;
}

export function captureMonth(item: Dated | undefined, clock: CaptureClock): string | null {
  return capturedAt(item, clock)?.toISOString().slice(0, 7) ?? null;
}

export function scrubModel(counts: readonly MonthCount[], direction: SortDir): ScrubModel {
  const known = counts.filter((entry) => MONTH_KEY.test(entry.month) && entry.count > 0);
  const ordered = [...known].sort((a, b) =>
    direction === 'desc' ? b.month.localeCompare(a.month) : a.month.localeCompare(b.month)
  );
  const total = ordered.reduce((sum, entry) => sum + entry.count, 0);
  let seen = 0;
  const months = ordered.map(({ month, count }) => {
    const start = seen / total;
    seen += count;
    return { month, year: Number(month.slice(0, 4)), count, start, end: seen / total };
  });
  return { months, total };
}

export function monthAt(model: ScrubModel, fraction: number): ScrubPoint | null {
  const { months } = model;
  if (months.length === 0) return null;
  const at = Math.min(1, Math.max(0, fraction));
  let low = 0;
  let high = months.length - 1;
  while (low < high) {
    const middle = Math.floor((low + high) / 2);
    const month = months[middle];
    if (month && month.end <= at) low = middle + 1;
    else high = middle;
  }
  const month = months[low];
  if (!month) return null;
  const span = month.end - month.start;
  return { index: low, within: span > 0 ? Math.min(1, Math.max(0, (at - month.start) / span)) : 0 };
}

export function fractionOf(model: ScrubModel, point: ScrubPoint): number {
  const month = model.months[point.index];
  if (!month) return 0;
  return month.start + (month.end - month.start) * Math.min(1, Math.max(0, point.within));
}

export function yearMarks(model: ScrubModel): ScrubMark[] {
  const marks: ScrubMark[] = [];
  let year: number | null = null;
  for (const month of model.months) {
    if (month.year === year) continue;
    year = month.year;
    marks.push({ label: String(year), at: month.start });
  }
  return marks;
}

export function spaced<T extends { at: number }>(
  marks: readonly T[],
  height: number,
  gap: number
): T[] {
  const kept: T[] = [];
  let last = Number.NEGATIVE_INFINITY;
  for (const mark of marks) {
    const y = mark.at * height;
    if (y - last < gap) continue;
    kept.push(mark);
    last = y;
  }
  return kept;
}

export type RailMarks = { years: ScrubMark[]; dots: number[] };

export function railMarks(
  model: ScrubModel,
  height: number,
  yearGap: number,
  dotGap: number
): RailMarks {
  const years = spaced(yearMarks(model), height, yearGap);
  const labels = years.map((mark) => mark.at * height);
  const dots: number[] = [];
  let last = Number.NEGATIVE_INFINITY;
  model.months.forEach((month, index) => {
    if (month.year !== model.months[index - 1]?.year) return;
    const y = month.start * height;
    if (y - last < dotGap) return;
    if (labels.some((label) => y > label - yearGap && y < label + dotGap / 2)) return;
    dots.push(month.start);
    last = y;
  });
  return { years, dots };
}

export function stepMonth(model: ScrubModel, index: number, delta: number): number {
  return Math.min(model.months.length - 1, Math.max(0, index + delta));
}

export function stepYear(model: ScrubModel, index: number, delta: number): number {
  const starts: number[] = [];
  model.months.forEach((month, at) => {
    if (month.year !== model.months[at - 1]?.year) starts.push(at);
  });
  let group = 0;
  starts.forEach((start, at) => {
    if (start <= index) group = at;
  });
  const target = starts[Math.min(starts.length - 1, Math.max(0, group + delta))];
  return target ?? index;
}

export function monthLabel(month: string): string {
  const match = MONTH_KEY.exec(month);
  if (!match) return month;
  const date = new Date(Date.UTC(Number(match[1]), Number(match[2]) - 1, 1));
  return date.toLocaleDateString('en-GB', { timeZone: 'UTC', month: 'long', year: 'numeric' });
}

export function photoOffset(model: ScrubModel, point: ScrubPoint): number {
  const month = model.months[point.index];
  if (!month) return 0;
  return Math.min(month.count - 1, Math.max(0, Math.floor(point.within * month.count)));
}

function originalsBetween(items: readonly Dated[], from: number, to: number): number {
  return items.slice(from, to).filter((item) => !item.copyOf).length;
}

function monthBase(
  items: readonly Dated[],
  start: number,
  hasPrevious: boolean,
  anchor: MonthAnchor | null,
  clock: CaptureClock
): number | null {
  if (start > 0 || !hasPrevious) return 0;
  const at = anchor ? items.findIndex((item) => item.id === anchor.id) : -1;
  if (!anchor || at < start || captureMonth(items[at], clock) !== anchor.month) return null;
  return anchor.offset - originalsBetween(items, start, at);
}

export function loadedPhotoAt(
  items: readonly Dated[],
  month: string,
  offset: number,
  hasPrevious: boolean,
  anchor: MonthAnchor | null,
  clock: CaptureClock
): LoadedPhoto | null {
  const start = items.findIndex((item) => captureMonth(item, clock) === month);
  if (start < 0) return null;
  const base = monthBase(items, start, hasPrevious, anchor?.month === month ? anchor : null, clock);
  if (base === null) return null;
  let seen = base;
  let last: string | null = null;
  for (const item of items.slice(start)) {
    if (captureMonth(item, clock) !== month) return last ? { id: last, exact: true } : null;
    if (item.copyOf) continue;
    if (seen >= offset) return { id: item.id, exact: seen === offset };
    seen += 1;
    last = item.id;
  }
  return last ? { id: last, exact: false } : null;
}

export function visibleMonth(
  model: ScrubModel,
  items: readonly Dated[],
  topIndex: number,
  hasPrevious: boolean,
  anchor: MonthAnchor | null,
  clock: CaptureClock
): ScrubPoint | null {
  const month = captureMonth(items[topIndex], clock);
  const index = model.months.findIndex((entry) => entry.month === month);
  const entry = model.months[index];
  if (!entry || !month) return null;
  let start = topIndex;
  while (start > 0 && captureMonth(items[start - 1], clock) === month) start -= 1;
  const base =
    monthBase(items, start, hasPrevious, anchor?.month === month ? anchor : null, clock) ?? 0;
  const before = base + originalsBetween(items, start, topIndex);
  return { index, within: Math.min(1, Math.max(0, before / entry.count)) };
}
