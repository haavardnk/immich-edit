import { describe, expect, it } from 'vitest';
import { dayAt, localDays, ordered, scrubDays, scrubLabel } from './scrubDays';

describe('localDays', () => {
  it('splits a UTC month at local midnights from its first to its last instant', () => {
    const days = localDays('2024-02');
    expect(days?.edges[0]).toBe('2024-02-01T00:00:00.000Z');
    expect(days?.edges.at(-1)).toBe('2024-03-01T00:00:00.000Z');
    expect(days?.edges).toHaveLength((days?.keys.length ?? 0) + 1);
    expect(days?.keys.length).toBeGreaterThanOrEqual(29);
    expect(days?.keys.length).toBeLessThanOrEqual(30);
    expect(new Set(days?.keys).size).toBe(days?.keys.length);
    const edges = days?.edges.map((edge) => Date.parse(edge)) ?? [];
    expect(edges.every((edge, at) => at === 0 || edge > (edges[at - 1] ?? 0))).toBe(true);
  });

  it('rejects a malformed month', () => {
    expect(localDays('2024-2')).toBeNull();
  });
});

describe('day counts', () => {
  const days = scrubDays(['2024-02-01', '2024-02-02', '2024-02-03'], [3, 0, 2]);

  it('drops empty days and orders them with the sort', () => {
    expect(days).toEqual([
      { key: '2024-02-01', count: 3 },
      { key: '2024-02-03', count: 2 }
    ]);
    expect(ordered(days, 'desc').map((day) => day.key)).toEqual(['2024-02-03', '2024-02-01']);
  });

  it('finds the day holding a photo offset', () => {
    expect(dayAt(days, 0)?.key).toBe('2024-02-01');
    expect(dayAt(days, 2)?.key).toBe('2024-02-01');
    expect(dayAt(days, 3)?.key).toBe('2024-02-03');
    expect(dayAt(days, 40)?.key).toBe('2024-02-03');
    expect(dayAt([], 0)).toBeNull();
  });

  it('labels a day when known and the month otherwise', () => {
    const month = { month: '2024-02', year: 2024, count: 5, start: 0, end: 1 };
    expect(scrubLabel({ key: '2024-02-03', count: 2 }, month)).toEqual({
      title: 'Saturday 3 February 2024',
      count: '2 photos'
    });
    expect(scrubLabel({ key: '2024-02-03', count: null }, month)?.count).toBeNull();
    expect(scrubLabel(null, month)).toEqual({ title: 'February 2024', count: '5 photos' });
    expect(scrubLabel(null, null)).toBeNull();
  });
});
