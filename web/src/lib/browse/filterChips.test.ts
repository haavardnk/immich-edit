import { describe, expect, it } from 'vitest';
import { activeFilterChips, photoCountLabel, withoutFilter } from './filterChips';
import { FILTER_DEFAULTS, type BrowseFilters } from '$lib/stores/browseControls.svelte';

const NAMES: Record<string, string> = { a1: 'Summer', p1: 'Ann', p2: 'Bo', t1: 'Trips/Rome' };

function chipsOf(filters: BrowseFilters) {
  return activeFilterChips(filters, (_, id) => NAMES[id] ?? 'Unknown person');
}

describe('browse filter chips', () => {
  it('names nothing for the default filters', () => {
    expect(chipsOf(FILTER_DEFAULTS)).toEqual([]);
  });

  it('names every active filter in a stable order', () => {
    const chips = chipsOf({
      favoriteOnly: true,
      notInAlbum: true,
      untagged: true,
      noPeople: true,
      albums: { ids: ['a1'], match: 'none' },
      people: { ids: ['p1', 'p2'], match: 'all' },
      tags: { ids: ['t1'], match: 'any' },
      rating: 3,
      filename: 'DSC',
      minSize: 25,
      maxSize: 100,
      visibility: 'archive',
      takenAfter: '2024-01-01',
      takenBefore: '2024-12-31',
      addedAfter: '2025-01-01',
      addedBefore: '2025-02-01',
      rejected: 'hide',
      label: 'green',
      make: ['SONY'],
      model: [],
      lensModel: ['FE 35mm F1.8', 'FE 85mm F1.8', 'XF35mmF1.4 R'],
      country: ['Norway'],
      state: [],
      city: ['Bergen', 'Oslo']
    });
    expect(chips.map((c) => c.label)).toEqual([
      'Archived',
      '3 stars',
      'Favorites',
      'Not in any album',
      'Untagged',
      'No people',
      'Without albums: Summer',
      'People: Ann +1',
      'Any of tags: Trips/Rome',
      'No rejected',
      'Green label',
      'Name: DSC',
      '≥ 25 MB',
      '≤ 100 MB',
      'After 2024-01-01',
      'Before 2024-12-31',
      'Added after 2025-01-01',
      'Added before 2025-02-01',
      'Make: SONY',
      'Lens: FE 35mm F1.8 +2',
      'Country: Norway',
      'City: Bergen +1'
    ]);
  });

  it('names the no-label filter', () => {
    expect(chipsOf({ ...FILTER_DEFAULTS, label: 'none' })[0]?.label).toBe('No label');
    expect(chipsOf({ ...FILTER_DEFAULTS, label: 'none' })[0]?.color).toBeUndefined();
    expect(chipsOf({ ...FILTER_DEFAULTS, label: 'blue' })[0]?.color).toBe('blue');
  });

  it.each([
    [1, '1 star'],
    ['3+', '≥ 3 stars'],
    ['1-', '≤ 1 star'],
    ['unrated', 'Unrated']
  ] as const)('labels rating %s as %s', (rating, label) => {
    expect(chipsOf({ ...FILTER_DEFAULTS, rating })[0]?.label).toBe(label);
  });

  it.each([
    ['hide', 'No rejected'],
    ['only', 'Rejected only']
  ] as const)('labels rejected %s as %s', (rejected, label) => {
    expect(chipsOf({ ...FILTER_DEFAULTS, rejected })[0]?.label).toBe(label);
  });

  it.each([
    ['any', ['p2'], 'Any of people: Bo'],
    ['none', ['p1', 'gone'], 'Without people: Ann +1'],
    ['all', ['gone'], 'People: Unknown person']
  ] as const)('labels people matching %s %j as %s', (match, ids, label) => {
    expect(chipsOf({ ...FILTER_DEFAULTS, people: { ids: [...ids], match } })[0]?.label).toBe(label);
  });

  it('removes one filter and keeps the rest', () => {
    const filters = { ...FILTER_DEFAULTS, rating: 4 as const, rejected: 'only' as const };
    expect(withoutFilter(filters, 'rating')).toEqual({ ...FILTER_DEFAULTS, rejected: 'only' });
  });

  it.each([
    [1, 0, '1 photo'],
    [1000, 0, '1000 photos'],
    [1000, 12, '1000 photos, 12 hidden']
  ])('counts %i photos with %i hidden', (total, hidden, label) => {
    expect(photoCountLabel(total, hidden)).toBe(label);
  });
});
