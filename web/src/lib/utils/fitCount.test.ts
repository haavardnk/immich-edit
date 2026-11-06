import { fitCount } from '#lib/utils/fitCount.ts';
import { describe, expect, it } from 'vitest';

describe('fitCount', () => {
  it('shows every item when they all fit', () => {
    expect(fitCount([40, 50, 60], 158, 4, 30)).toBe(3);
  });

  it('keeps room for the overflow item when some are hidden', () => {
    expect(fitCount([40, 50, 60], 157, 4, 30)).toBe(2);
    expect(fitCount([40, 50, 60], 127, 4, 30)).toBe(1);
  });

  it('always shows the first item so it can truncate', () => {
    expect(fitCount([200, 50], 100, 4, 30)).toBe(1);
    expect(fitCount([200], 100, 4, 30)).toBe(1);
  });

  it('shows nothing without items', () => {
    expect(fitCount([], 100, 4, 30)).toBe(0);
  });
});
