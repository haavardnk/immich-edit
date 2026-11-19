import type { CullState } from '#lib/browse/cullState.ts';
import { describe, expect, it } from 'vitest';
import { markText } from './cullMark';

const CLEAR: CullState = {
  rating: 0,
  isFavorite: false,
  rejected: false,
  label: null,
  owned: true
};

describe('markText', () => {
  it.each([
    [0, 'No rating'],
    [1, '1 star'],
    [3, '3 stars']
  ])('reads a rating of %i as %s', (rating, text) => {
    expect(markText('rating', { ...CLEAR, rating })).toBe(text);
  });

  it('reads flags on and off', () => {
    expect(markText('favorite', { ...CLEAR, isFavorite: true })).toBe('Favorite');
    expect(markText('favorite', CLEAR)).toBe('Not a favorite');
    expect(markText('reject', { ...CLEAR, rejected: true })).toBe('Rejected');
    expect(markText('reject', CLEAR)).toBe('Not rejected');
  });

  it('names the label colour', () => {
    expect(markText('label', { ...CLEAR, label: 'green' })).toBe('Green label');
    expect(markText('label', CLEAR)).toBe('No label');
  });
});
