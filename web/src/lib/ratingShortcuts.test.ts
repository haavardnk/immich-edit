import { describe, expect, it } from 'vitest';
import { nextRatingFromKey } from './ratingShortcuts';

describe('nextRatingFromKey', () => {
  it.each([
    ['a', 3, undefined],
    ['Enter', null, undefined],
    [' ', 2, undefined],
    ['6', null, undefined],
    ['0', 4, null],
    ['0', null, null],
    ['3', null, 3],
    ['3', 0, 3],
    ['3', 3, null],
    ['3', 2, 3],
    ['5', undefined, 5],
    ['1', undefined, 1]
  ])('key %s with current %s -> %s', (key, current, expected) => {
    expect(nextRatingFromKey(key, current as number | null | undefined)).toBe(expected);
  });
});
