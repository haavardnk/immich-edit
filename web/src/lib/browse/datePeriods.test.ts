import { describe, expect, it } from 'vitest';
import {
  calendarYears,
  dateChipLabel,
  datePeriod,
  isDateKey,
  loadedMonths,
  monthOpen,
  monthRange,
  moveDateRange,
  pairedDateKey,
  periodLabel,
  selectedBasis,
  todayDay,
  withDateRange,
  withoutDates,
  yearRange
} from './datePeriods';
import type { AssetSummary } from '#lib/types/album.ts';
import { FILTER_DEFAULTS } from '#lib/stores/browseControls.svelte.ts';

const TODAY = new Date(2026, 9, 24);

describe('date periods', () => {
  it('spans whole years and months', () => {
    expect(yearRange(2024)).toEqual({ after: '2024-01-01', before: '2024-12-31' });
    expect(monthRange(2024, 2)).toEqual({ after: '2024-02-01', before: '2024-02-29' });
    expect(monthRange(2026, 11)).toEqual({ after: '2026-11-01', before: '2026-11-30' });
  });

  it.each([
    [{ after: '', before: '' }, { kind: 'any' }],
    [
      { after: '2025-01-01', before: '2025-12-31' },
      { kind: 'year', year: 2025 }
    ],
    [
      { after: '2025-03-01', before: '2025-03-31' },
      { kind: 'month', year: 2025, month: 3 }
    ],
    [{ after: '2025-03-02', before: '2025-03-31' }, { kind: 'custom' }],
    [{ after: '', before: '2025-03-31' }, { kind: 'custom' }]
  ])('reads %j as %j', (range, period) => {
    expect(datePeriod(range)).toEqual(period);
  });

  it.each([
    [{ after: '', before: '' }, 'Any date'],
    [{ after: '2025-01-01', before: '2025-12-31' }, '2025'],
    [{ after: '2025-03-01', before: '2025-03-31' }, 'Mar 2025'],
    [{ after: '2026-10-24', before: '' }, 'From 24 Oct 2026'],
    [{ after: '', before: '2026-10-24' }, 'Until 24 Oct 2026'],
    [{ after: '2026-10-02', before: '2026-10-24' }, '2 Oct 2026 – 24 Oct 2026']
  ])('labels %j as %s', (range, label) => {
    expect(periodLabel(range)).toBe(label);
  });

  it('prefixes added dates', () => {
    expect(dateChipLabel(yearRange(2025), 'added')).toBe('Added 2025');
    expect(dateChipLabel({ after: '2026-10-24', before: '' }, 'added')).toBe(
      'Added from 24 Oct 2026'
    );
    expect(dateChipLabel(yearRange(2025), 'taken')).toBe('2025');
  });

  it('moves a range between taken and added', () => {
    const taken = withDateRange(FILTER_DEFAULTS, 'taken', yearRange(2025));
    expect(selectedBasis(taken, 'added')).toBe('taken');
    expect(moveDateRange(taken, 'taken', 'added')).toEqual({
      ...FILTER_DEFAULTS,
      addedAfter: '2025-01-01',
      addedBefore: '2025-12-31'
    });
    expect(selectedBasis(FILTER_DEFAULTS, 'added')).toBe('added');
  });

  it('pairs the ends of each range', () => {
    expect(pairedDateKey('takenAfter')).toBe('takenBefore');
    expect(pairedDateKey('addedBefore')).toBe('addedAfter');
    expect(isDateKey('addedAfter')).toBe(true);
    expect(isDateKey('minSize')).toBe(false);
  });

  it('lists the years and months that have photos taken', () => {
    const years = calendarYears(
      [
        { month: '2026-10', count: 4 },
        { month: '2026-03', count: 1 },
        { month: '2019-07', count: 2 },
        { month: '2018-01', count: 0 }
      ],
      'taken',
      TODAY,
      null
    );
    expect(years.map((entry) => entry.year)).toEqual([2026, 2019]);
    expect([...(years[0]?.months ?? [])].sort((a, b) => a - b)).toEqual([3, 10]);
  });

  it('lists every year back to the oldest known for added dates', () => {
    const years = calendarYears([{ month: '2023-05', count: 1 }], 'added', TODAY, null);
    expect(years.map((entry) => entry.year)).toEqual([2026, 2025, 2024, 2023]);
    expect(years[0]?.months).toBeNull();
  });

  it('lists the last twenty years when nothing is known', () => {
    const years = calendarYears(null, 'taken', TODAY, null);
    expect(years).toHaveLength(20);
    expect(years.at(-1)?.year).toBe(2007);
    expect(years[0]?.months).toBeNull();
  });

  it('lists no years while taken months are empty', () => {
    expect(calendarYears([], 'taken', TODAY, null)).toEqual([]);
  });

  it('counts loaded photos by local month', () => {
    const months = loadedMonths([
      { fileCreatedAt: new Date(2026, 9, 3).toISOString() },
      { fileCreatedAt: new Date(2026, 9, 30).toISOString() },
      { fileCreatedAt: null }
    ] as AssetSummary[]);
    expect(months).toEqual([{ month: '2026-10', count: 2 }]);
  });

  it('clears both date ranges', () => {
    const filters = withDateRange(
      withDateRange(FILTER_DEFAULTS, 'taken', yearRange(2025)),
      'added',
      yearRange(2026)
    );
    expect(withoutDates({ ...filters, rating: 3 })).toEqual({ ...FILTER_DEFAULTS, rating: 3 });
  });

  it('keeps a selected year without photos', () => {
    const years = calendarYears([{ month: '2026-10', count: 4 }], 'taken', TODAY, 2010);
    expect(years.map((entry) => entry.year)).toEqual([2026, 2010]);
  });

  it('closes future months and months without photos', () => {
    expect(monthOpen(2026, 10, null, TODAY)).toBe(true);
    expect(monthOpen(2026, 11, null, TODAY)).toBe(false);
    expect(monthOpen(2027, 1, null, TODAY)).toBe(false);
    expect(monthOpen(2026, 3, new Set([10]), TODAY)).toBe(false);
    expect(monthOpen(2026, 10, new Set([10]), TODAY)).toBe(true);
  });

  it('formats today as a day', () => {
    expect(todayDay(TODAY)).toBe('2026-10-24');
  });
});
