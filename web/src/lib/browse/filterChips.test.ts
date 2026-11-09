import { describe, expect, it } from 'vitest';
import {
  activeFilterChips,
  activeFilterCount,
  activeFilterValues,
  filterValueLabel,
  photoCountLabel,
  showPhotosLabel,
  withoutFilter
} from './filterChips';
import { FILTER_DEFAULTS, type BrowseFilters } from '#lib/stores/browseControls.svelte.ts';

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
      '25–100 MB',
      '2024',
      'Added 1 Jan 2025 – 1 Feb 2025',
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

  it.each([
    ['visibility', 'Timeline'],
    ['rejected', 'Rejected shown'],
    ['rating', 'Any rating'],
    ['label', 'Any label']
  ] as const)('labels the default %s as %s', (key, label) => {
    expect(filterValueLabel(FILTER_DEFAULTS, key, () => '')).toBe(label);
  });

  it('names active values without their field where the value says enough', () => {
    const values = activeFilterValues(
      {
        ...FILTER_DEFAULTS,
        rating: '3+',
        people: { ids: ['p1'], match: 'all' },
        tags: { ids: ['t1'], match: 'none' },
        model: ['X-T5'],
        country: ['Norway']
      },
      (_, id) => NAMES[id] ?? 'Unknown'
    );
    expect(values).toEqual(['≥ 3 stars', 'Ann', 'Without tags: Trips/Rome', 'X-T5', 'Norway']);
  });

  it('names no values for the default filters', () => {
    expect(activeFilterValues(FILTER_DEFAULTS, () => '')).toEqual([]);
  });

  it.each<[Partial<BrowseFilters>, string]>([
    [{ takenAfter: '2026-10-24' }, 'From 24 Oct 2026'],
    [{ takenBefore: '2026-10-24' }, 'Until 24 Oct 2026'],
    [{ takenAfter: '2026-10-01', takenBefore: '2026-10-31' }, 'Oct 2026'],
    [{ addedAfter: '2026-10-24' }, 'Added from 24 Oct 2026'],
    [{ maxSize: 1 }, '< 1 MB'],
    [{ minSize: 50 }, '> 50 MB'],
    [{ minSize: 25 }, '≥ 25 MB'],
    [{ maxSize: 100 }, '≤ 100 MB']
  ])('pairs range ends %j into one chip %s', (filters, label) => {
    expect(chipsOf({ ...FILTER_DEFAULTS, ...filters }).map((chip) => chip.label)).toEqual([label]);
  });

  it('counts a paired range once', () => {
    expect(
      activeFilterCount({
        ...FILTER_DEFAULTS,
        takenAfter: '2026-10-01',
        takenBefore: '2026-10-31',
        minSize: 1,
        maxSize: 10,
        rating: 5
      })
    ).toBe(3);
    expect(activeFilterCount(FILTER_DEFAULTS)).toBe(0);
  });

  it('removes both ends of a paired range', () => {
    const filters = { ...FILTER_DEFAULTS, minSize: 1, maxSize: 10, rating: 2 as const };
    expect(withoutFilter(filters, 'minSize')).toEqual({ ...FILTER_DEFAULTS, rating: 2 });
    expect(withoutFilter({ ...FILTER_DEFAULTS, takenBefore: '2026-01-01' }, 'takenBefore')).toEqual(
      FILTER_DEFAULTS
    );
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

  it.each([
    [undefined, 'Show photos'],
    [0, 'Show 0 photos'],
    [1, 'Show 1 photo'],
    [1204, 'Show 1204 photos']
  ])('labels the filter sheet button for %s photos as %s', (total, label) => {
    expect(showPhotosLabel(total)).toBe(label);
  });
});
