import type { BrowseFilters } from '#lib/stores/browseControls.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import type { MonthCount } from '#lib/types/search.ts';

export type DateBasis = 'taken' | 'added';

export const DATE_BASES: readonly DateBasis[] = ['taken', 'added'];

export const DATE_BASIS_LABELS: Readonly<Record<DateBasis, string>> = {
  taken: 'Taken',
  added: 'Added'
};

export const DATE_KEYS = {
  taken: { after: 'takenAfter', before: 'takenBefore' },
  added: { after: 'addedAfter', before: 'addedBefore' }
} as const;

export type DateKey = (typeof DATE_KEYS)[DateBasis][keyof (typeof DATE_KEYS)[DateBasis]];

export interface DateRange {
  after: string;
  before: string;
}

export type DatePeriod =
  | { kind: 'any' }
  | { kind: 'year'; year: number }
  | { kind: 'month'; year: number; month: number }
  | { kind: 'custom' };

export const MONTHS = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12] as const;

const NO_RANGE: DateRange = { after: '', before: '' };
const UNKNOWN_YEARS = 20;
const DAY = /^(\d{4})-(\d{2})-(\d{2})$/;

function pad(value: number): string {
  return String(value).padStart(2, '0');
}

function lastDay(year: number, month: number): number {
  return new Date(Date.UTC(year, month, 0)).getUTCDate();
}

function utcDate(day: string): Date {
  return new Date(`${day}T00:00:00Z`);
}

const MONTH_NAMES = [
  'Jan',
  'Feb',
  'Mar',
  'Apr',
  'May',
  'Jun',
  'Jul',
  'Aug',
  'Sep',
  'Oct',
  'Nov',
  'Dec'
] as const;

export function monthName(month: number): string {
  return MONTH_NAMES[month - 1] ?? '';
}

export function dayLabel(day: string): string {
  const date = utcDate(day);
  return `${date.getUTCDate()} ${monthName(date.getUTCMonth() + 1)} ${date.getUTCFullYear()}`;
}

export function yearRange(year: number): DateRange {
  return { after: `${year}-01-01`, before: `${year}-12-31` };
}

export function monthRange(year: number, month: number): DateRange {
  const prefix = `${year}-${pad(month)}`;
  return { after: `${prefix}-01`, before: `${prefix}-${pad(lastDay(year, month))}` };
}

export function dateRange(filters: BrowseFilters, basis: DateBasis): DateRange {
  const keys = DATE_KEYS[basis];
  return { after: filters[keys.after], before: filters[keys.before] };
}

export function hasRange(range: DateRange): boolean {
  return range.after !== '' || range.before !== '';
}

export function datePeriod({ after, before }: DateRange): DatePeriod {
  if (!after && !before) return { kind: 'any' };
  const start = DAY.exec(after);
  if (!start) return { kind: 'custom' };
  const year = Number(start[1]);
  const month = Number(start[2]);
  const same = (range: DateRange): boolean => range.after === after && range.before === before;
  if (same(yearRange(year))) return { kind: 'year', year };
  if (same(monthRange(year, month))) return { kind: 'month', year, month };
  return { kind: 'custom' };
}

export function selectedBasis(filters: BrowseFilters, fallback: DateBasis): DateBasis {
  if (hasRange(dateRange(filters, fallback))) return fallback;
  return DATE_BASES.find((basis) => hasRange(dateRange(filters, basis))) ?? fallback;
}

export function withDateRange(
  filters: BrowseFilters,
  basis: DateBasis,
  range: DateRange
): BrowseFilters {
  const keys = DATE_KEYS[basis];
  return { ...filters, [keys.after]: range.after, [keys.before]: range.before };
}

export function moveDateRange(
  filters: BrowseFilters,
  from: DateBasis,
  to: DateBasis
): BrowseFilters {
  if (from === to) return filters;
  return withDateRange(withDateRange(filters, to, dateRange(filters, from)), from, NO_RANGE);
}

export function periodLabel(range: DateRange): string {
  const period = datePeriod(range);
  if (period.kind === 'any') return 'Any date';
  if (period.kind === 'year') return String(period.year);
  if (period.kind === 'month') return `${monthName(period.month)} ${period.year}`;
  if (range.after && range.before) return `${dayLabel(range.after)} – ${dayLabel(range.before)}`;
  if (range.after) return `From ${dayLabel(range.after)}`;
  return `Until ${dayLabel(range.before)}`;
}

export function dateChipLabel(range: DateRange, basis: DateBasis): string {
  const label = periodLabel(range);
  if (basis === 'taken') return label;
  return `Added ${label.charAt(0).toLowerCase()}${label.slice(1)}`;
}

export function pairedDateKey(key: DateKey): DateKey {
  for (const keys of Object.values(DATE_KEYS)) {
    if (keys.after === key) return keys.before;
    if (keys.before === key) return keys.after;
  }
  return key;
}

export function isDateKey(key: string): key is DateKey {
  return Object.values(DATE_KEYS).some((keys) => keys.after === key || keys.before === key);
}

export interface CalendarYear {
  year: number;
  months: ReadonlySet<number> | null;
}

function takenMonths(counts: readonly MonthCount[]): Map<number, Set<number>> {
  const byYear = new Map<number, Set<number>>();
  for (const { month, count } of counts) {
    const match = /^(\d{4})-(\d{2})$/.exec(month);
    if (!match || count <= 0) continue;
    const year = Number(match[1]);
    const months = byYear.get(year) ?? new Set<number>();
    months.add(Number(match[2]));
    byYear.set(year, months);
  }
  return byYear;
}

export function calendarYears(
  counts: readonly MonthCount[] | null,
  basis: DateBasis,
  today: Date,
  selected: number | null
): CalendarYear[] {
  const taken = takenMonths(counts ?? []);
  const current = today.getFullYear();
  const known = basis === 'taken' && counts !== null;
  const years = new Set<number>(known ? taken.keys() : []);
  if (!known) {
    const earliest =
      counts === null ? current - UNKNOWN_YEARS + 1 : Math.min(current, ...taken.keys());
    for (let year = current; year >= earliest; year -= 1) years.add(year);
  }
  if (selected !== null) years.add(selected);
  return [...years]
    .sort((a, b) => b - a)
    .map((year) => ({ year, months: known ? (taken.get(year) ?? new Set()) : null }));
}

export function monthOpen(
  year: number,
  month: number,
  known: ReadonlySet<number> | null,
  today: Date
): boolean {
  const future =
    year > today.getFullYear() || (year === today.getFullYear() && month > today.getMonth() + 1);
  if (future) return false;
  return known === null || known.has(month);
}

export function withoutDates(filters: BrowseFilters): BrowseFilters {
  return withDateRange(withDateRange(filters, 'taken', NO_RANGE), 'added', NO_RANGE);
}

export function loadedMonths(assets: readonly AssetSummary[]): MonthCount[] {
  const counts = new Map<string, number>();
  for (const asset of assets) {
    if (!asset.fileCreatedAt) continue;
    const date = new Date(asset.fileCreatedAt);
    if (Number.isNaN(date.getTime())) continue;
    const month = `${date.getFullYear()}-${pad(date.getMonth() + 1)}`;
    counts.set(month, (counts.get(month) ?? 0) + 1);
  }
  return [...counts].map(([month, count]) => ({ month, count }));
}

export function todayDay(today: Date): string {
  return `${today.getFullYear()}-${pad(today.getMonth() + 1)}-${pad(today.getDate())}`;
}
