import { describe, expect, it } from 'vitest';
import {
  boundEnd,
  boundStart,
  holdsWallClock,
  joinBound,
  localSpan,
  readBound
} from './dateBounds';

describe('date bounds', () => {
  it.each([
    ['2025-03-18', '2025-03-18T00:00:00', '2025-03-19T00:00:00'],
    ['2025-03-18T06:30', '2025-03-18T06:30:00', '2025-03-18T06:31:00'],
    ['2025-12-31T23:59', '2025-12-31T23:59:00', '2026-01-01T00:00:00'],
    ['2024-02-28', '2024-02-28T00:00:00', '2024-02-29T00:00:00']
  ])('bounds %s from %s until %s', (bound, start, end) => {
    expect([boundStart(bound), boundEnd(bound)]).toEqual([start, end]);
  });

  it('sends only the ends that are set', () => {
    expect(localSpan('', '')).toBeUndefined();
    expect(localSpan('2025-03-18', '')).toEqual({ from: '2025-03-18T00:00:00' });
    expect(localSpan('', '2025-03-18')).toEqual({ until: '2025-03-19T00:00:00' });
  });

  it('holds a wall clock from the start until before the end', () => {
    const day = { from: '2025-03-18T00:00:00', until: '2025-03-19T00:00:00' };
    expect(holdsWallClock(day, '2025-03-18T00:00:00')).toBe(true);
    expect(holdsWallClock(day, '2025-03-18T23:59:59')).toBe(true);
    expect(holdsWallClock(day, '2025-03-19T00:00:00')).toBe(false);
    expect(holdsWallClock(day, '2025-03-17T23:59:59')).toBe(false);
    expect(holdsWallClock(day, '')).toBe(false);
  });

  it('joins a day and an optional time and drops a time without a day', () => {
    expect(joinBound('2025-03-18', '')).toBe('2025-03-18');
    expect(joinBound('2025-03-18', '06:30')).toBe('2025-03-18T06:30');
    expect(joinBound('', '06:30')).toBe('');
  });

  it.each([
    ['2025-03-18', '2025-03-18'],
    ['2025-03-18T06:30', '2025-03-18T06:30'],
    ['2025-02-30', ''],
    ['2025-03-18T24:00', ''],
    ['2025-03-18T06:30:00', ''],
    [null, '']
  ])('reads %s as %j', (value, read) => {
    expect(readBound(value)).toBe(read);
  });
});
