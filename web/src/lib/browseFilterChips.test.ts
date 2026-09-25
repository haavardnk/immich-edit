import { describe, expect, it } from 'vitest';
import { activeFilterChips, photoCountLabel, withoutFilter } from './browseFilterChips';
import { FILTER_DEFAULTS } from '$lib/stores/browseControls.svelte';

describe('browse filter chips', () => {
  it('names nothing for the default filters', () => {
    expect(activeFilterChips(FILTER_DEFAULTS)).toEqual([]);
  });

  it('names every active filter in a stable order', () => {
    const chips = activeFilterChips({
      favoriteOnly: true,
      rating: 3,
      filename: 'DSC',
      visibility: 'archive',
      takenAfter: '2024-01-01',
      takenBefore: '2024-12-31',
      excludeRejected: true
    });
    expect(chips.map((c) => c.label)).toEqual([
      'Archived',
      '3 stars',
      'Favorites',
      'No rejected',
      'Name: DSC',
      'After 2024-01-01',
      'Before 2024-12-31'
    ]);
  });

  it.each([
    [1, '1 star'],
    ['unrated', 'Unrated']
  ] as const)('labels rating %s as %s', (rating, label) => {
    expect(activeFilterChips({ ...FILTER_DEFAULTS, rating })[0]?.label).toBe(label);
  });

  it('removes one filter and keeps the rest', () => {
    const filters = { ...FILTER_DEFAULTS, rating: 4 as const, excludeRejected: true };
    expect(withoutFilter(filters, 'rating')).toEqual({ ...FILTER_DEFAULTS, excludeRejected: true });
  });

  it.each([
    [1, 0, '1 photo'],
    [1000, 0, '1000 photos'],
    [1000, 12, '1000 photos, 12 hidden']
  ])('counts %i photos with %i hidden', (total, hidden, label) => {
    expect(photoCountLabel(total, hidden)).toBe(label);
  });
});
