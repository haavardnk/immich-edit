import { describe, expect, it } from 'vitest';
import {
  MAX_RECENT_SEARCHES,
  sameRecents,
  storedRecents,
  withoutRecent,
  withRecent
} from './recentSearches';

describe('withRecent', () => {
  it('puts a new query first and trims it', () => {
    expect(withRecent(['boats'], '  harbour ')).toEqual(['harbour', 'boats']);
  });

  it('moves a repeated query to the front without a duplicate', () => {
    expect(withRecent(['harbour', 'Boats', 'IMG_4120'], 'boats')).toEqual([
      'boats',
      'harbour',
      'IMG_4120'
    ]);
  });

  it('replaces the newest entry while one typing session grows or shrinks the query', () => {
    expect(withRecent(['har', 'boats'], 'harbour', true)).toEqual(['harbour', 'boats']);
    expect(withRecent(['harbour', 'boats'], 'harb', true)).toEqual(['harb', 'boats']);
  });

  it('keeps a prefix from an earlier session', () => {
    expect(withRecent(['red', 'boats'], 'red car')).toEqual(['red car', 'red', 'boats']);
  });

  it('ignores an empty query', () => {
    expect(withRecent(['boats'], '   ')).toEqual(['boats']);
  });

  it('keeps at most the limit', () => {
    const many = Array.from({ length: MAX_RECENT_SEARCHES }, (_, i) => `q${i}`);
    const next = withRecent(many, 'new');
    expect(next).toHaveLength(MAX_RECENT_SEARCHES);
    expect(next[0]).toBe('new');
    expect(next).not.toContain(`q${MAX_RECENT_SEARCHES - 1}`);
  });
});

describe('withoutRecent', () => {
  it('drops one entry', () => {
    expect(withoutRecent(['a', 'b'], 'a')).toEqual(['b']);
  });
});

describe('storedRecents', () => {
  it('keeps only non-empty strings from stored data', () => {
    expect(storedRecents(['a', 3, '', ' ', 'b'])).toEqual(['a', 'b']);
    expect(storedRecents('a')).toEqual([]);
    expect(storedRecents(undefined)).toEqual([]);
  });
});

describe('sameRecents', () => {
  it('compares entries in order', () => {
    expect(sameRecents(['a', 'b'], ['a', 'b'])).toBe(true);
    expect(sameRecents(['a', 'b'], ['b', 'a'])).toBe(false);
    expect(sameRecents(['a'], ['a', 'b'])).toBe(false);
  });
});
