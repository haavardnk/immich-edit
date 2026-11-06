import { describe, expect, it } from 'vitest';
import {
  anchorFilter,
  fractionOf,
  loadedMonthStart,
  monthAt,
  monthCountLabel,
  monthLabel,
  railMarks,
  scrubModel,
  spaced,
  stepMonth,
  stepYear,
  utcMonth,
  visibleMonth,
  yearMarks
} from './scrubberModel';

const counts = [
  { month: '2026-10', count: 50 },
  { month: '2026-01', count: 25 },
  { month: '2025-07', count: 25 }
];

describe('scrubModel', () => {
  it('places months by their share of photos, newest first for desc', () => {
    const model = scrubModel(counts, 'desc');
    expect(model.total).toBe(100);
    expect(model.months.map(({ month, start, end }) => [month, start, end])).toEqual([
      ['2026-10', 0, 0.5],
      ['2026-01', 0.5, 0.75],
      ['2025-07', 0.75, 1]
    ]);
  });

  it('reverses for oldest first and drops empty or malformed months', () => {
    const model = scrubModel(
      [...counts, { month: '2024-02', count: 0 }, { month: 'x', count: 3 }],
      'asc'
    );
    expect(model.months.map((month) => month.month)).toEqual(['2025-07', '2026-01', '2026-10']);
    expect(model.months[0]?.end).toBe(0.25);
  });
});

describe('monthAt and fractionOf', () => {
  const model = scrubModel(counts, 'desc');

  it.each([
    [0, 0, 0],
    [0.25, 0, 0.5],
    [0.5, 1, 0],
    [0.875, 2, 0.5],
    [1.4, 2, 1],
    [-1, 0, 0]
  ])('maps %f to month %i at %f', (fraction, index, within) => {
    expect(monthAt(model, fraction)).toEqual({ index, within });
  });

  it('maps a month position back to its fraction', () => {
    expect(fractionOf(model, { index: 1, within: 0.5 })).toBe(0.625);
    expect(fractionOf(model, { index: 9, within: 0 })).toBe(0);
  });

  it('has no month without counts', () => {
    expect(monthAt(scrubModel([], 'desc'), 0.5)).toBeNull();
  });
});

describe('year marks', () => {
  it('marks where each year starts', () => {
    expect(yearMarks(scrubModel(counts, 'desc'))).toEqual([
      { label: '2026', at: 0 },
      { label: '2025', at: 0.75 }
    ]);
  });

  it('drops marks that would overlap at the rail height', () => {
    const marks = [{ at: 0 }, { at: 0.01 }, { at: 0.2 }];
    expect(spaced(marks, 500, 20)).toEqual([{ at: 0 }, { at: 0.2 }]);
  });
});

describe('railMarks', () => {
  it('puts a dot on every later month of a year, apart from the labels and each other', () => {
    const model = scrubModel(
      [
        { month: '2026-03', count: 10 },
        { month: '2026-02', count: 10 },
        { month: '2026-01', count: 1 },
        { month: '2025-12', count: 9 },
        { month: '2025-11', count: 10 }
      ],
      'desc'
    );
    const marks = railMarks(model, 400, 24, 10);
    expect(marks.years.map((mark) => mark.label)).toEqual(['2026', '2025']);
    expect(marks.dots).toEqual([0.25, 0.75]);
    expect(railMarks(model, 40, 24, 10).years.map((mark) => mark.label)).toEqual(['2026']);
  });
});

describe('keyboard steps', () => {
  const model = scrubModel(
    [
      { month: '2026-10', count: 1 },
      { month: '2026-09', count: 1 },
      { month: '2025-12', count: 1 },
      { month: '2025-01', count: 1 },
      { month: '2023-05', count: 1 }
    ],
    'desc'
  );

  it('steps one month and stops at the ends', () => {
    expect(stepMonth(model, 0, -1)).toBe(0);
    expect(stepMonth(model, 3, 1)).toBe(4);
    expect(stepMonth(model, 4, 1)).toBe(4);
  });

  it('steps to the first month of the next or previous year', () => {
    expect(stepYear(model, 1, 1)).toBe(2);
    expect(stepYear(model, 3, 1)).toBe(4);
    expect(stepYear(model, 3, -1)).toBe(0);
    expect(stepYear(model, 4, 1)).toBe(4);
  });
});

describe('labels', () => {
  it('names a month and its count', () => {
    expect(monthLabel('2025-07')).toBe('July 2025');
    expect(monthCountLabel({ count: 26 })).toBe('26 photos');
    expect(monthCountLabel({ count: 1 })).toBe('1 photo');
  });

  it('reads the UTC month of fileCreatedAt', () => {
    expect(utcMonth('2025-07-31T23:30:00-02:00')).toBe('2025-08');
    expect(utcMonth(null)).toBeNull();
    expect(utcMonth('nope')).toBeNull();
  });
});

describe('loaded photos', () => {
  const items = [
    { id: 'a', fileCreatedAt: '2026-10-05T00:00:00Z' },
    { id: 'b', fileCreatedAt: '2026-10-01T00:00:00Z' },
    { id: 'b_1', fileCreatedAt: '2026-10-01T00:00:00Z', copyOf: 'b' },
    { id: 'c', fileCreatedAt: '2026-01-10T00:00:00Z' },
    { id: 'd', fileCreatedAt: '2026-01-02T00:00:00Z' }
  ];

  it('finds the first loaded photo of a month', () => {
    expect(loadedMonthStart(items, '2026-01', false)).toBe('c');
    expect(loadedMonthStart(items, '2025-07', false)).toBeNull();
  });

  it('does not trust a month cut off at the near end of a window', () => {
    expect(loadedMonthStart(items, '2026-10', true)).toBeNull();
    expect(loadedMonthStart(items, '2026-10', false)).toBe('a');
  });

  it('marks the visible range inside its month', () => {
    const model = scrubModel(counts, 'desc');
    expect(visibleMonth(model, items, 0)).toEqual({ index: 0, within: 0 });
    expect(visibleMonth(model, items, 3)).toEqual({ index: 1, within: 0 });
    expect(visibleMonth(model, items, 4)).toEqual({ index: 1, within: 0.04 });
    expect(visibleMonth(model, items, 2)).toEqual({ index: 0, within: 0.04 });
    expect(visibleMonth(model, [{ id: 'x', fileCreatedAt: null }], 0)).toBeNull();
  });
});

describe('anchorFilter', () => {
  it('ends a newest-first search at the last moment of the month', () => {
    expect(anchorFilter({ favorite: true }, '2025-02', 'desc')).toEqual({
      favorite: true,
      takenBefore: '2025-02-28T23:59:59.999Z'
    });
  });

  it('starts an oldest-first search at the first moment of the month', () => {
    expect(anchorFilter({}, '2025-12', 'asc')).toEqual({ takenAfter: '2025-12-01T00:00:00.000Z' });
  });

  it('keeps a tighter date filter', () => {
    expect(
      anchorFilter({ takenBefore: '2025-02-10T00:00:00.000Z' }, '2025-02', 'desc').takenBefore
    ).toBe('2025-02-10T00:00:00.000Z');
    expect(
      anchorFilter({ takenAfter: '2025-12-20T00:00:00.000Z' }, '2025-12', 'asc').takenAfter
    ).toBe('2025-12-20T00:00:00.000Z');
  });
});
