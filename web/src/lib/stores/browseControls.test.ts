import { describe, it, expect, beforeEach, vi } from 'vitest';
import { BrowseControlsStore, browseControls, type SortFamily } from './browseControls.svelte';
import { immichCapabilities } from './immichCapabilities.svelte';
import { REJECT_TAG_VALUE } from '$lib/browse/reject';
import type { AssetSummary } from '$lib/types/album';
import type { ExifInfo } from '$lib/types/asset';

const SORT_KEY = 'immich-edit:browseSort';

const store = new Map<string, string>();
vi.stubGlobal('localStorage', {
  getItem: (key: string) => store.get(key) ?? null,
  setItem: (key: string, value: string) => store.set(key, value),
  removeItem: (key: string) => store.delete(key),
  clear: () => store.clear()
});

function asset(rejected: boolean, rating: number | null = null): AssetSummary {
  return {
    id: 'a1',
    originalFileName: 'a.jpg',
    type: 'IMAGE',
    fileCreatedAt: null,
    updatedAt: null,
    checksum: null,
    isFavorite: false,
    exifInfo: rating === null ? null : ({ rating } as ExifInfo),
    tags: rejected ? [{ id: 'r', name: 'reject', value: REJECT_TAG_VALUE }] : []
  };
}

describe('browseControls excludeRejected', () => {
  beforeEach(() => browseControls.reset());

  it('is off by default', () => {
    expect(browseControls.excludeRejected).toBe(false);
    expect(browseControls.isDefault).toBe(true);
    expect(browseControls.isFiltered).toBe(false);
  });

  it('counts as a filter without changing server query', () => {
    const keyBefore = browseControls.serverFilterKey();
    const filterBefore = browseControls.filter({});
    browseControls.excludeRejected = true;
    expect(browseControls.isDefault).toBe(false);
    expect(browseControls.isFiltered).toBe(true);
    expect(browseControls.serverFilterKey()).toBe(keyBefore);
    expect(browseControls.filter({})).toEqual(filterBefore);
    expect(browseControls.hides(asset(true))).toBe(true);
  });

  it('is cleared by reset', () => {
    browseControls.excludeRejected = true;
    browseControls.reset();
    expect(browseControls.excludeRejected).toBe(false);
  });
});

describe('browseControls sort families', () => {
  beforeEach(() => localStorage.clear());

  it.each([
    ['timeline', 'desc'],
    ['collection', 'asc'],
    ['edited', 'asc']
  ] as [SortFamily, 'asc' | 'desc'][])('defaults %s to %s', (family, dir) => {
    const controls = new BrowseControlsStore();
    controls.enter('ctx', family);
    expect(controls.sortDir).toBe(dir);
    expect(controls.isDefault).toBe(true);
  });

  it('keeps each family independent and survives a reload', () => {
    const controls = new BrowseControlsStore();
    controls.enter('photos', 'timeline');
    controls.setSortDir('asc');
    controls.enter('album:1', 'collection');
    expect(controls.sortDir).toBe('asc');
    controls.setSortDir('desc');
    controls.enter('photos', 'timeline');
    expect(controls.sortDir).toBe('asc');

    const reloaded = new BrowseControlsStore();
    reloaded.enter('album:1', 'collection');
    expect(reloaded.sortDir).toBe('desc');
    reloaded.enter('edited', 'edited');
    expect(reloaded.sortDir).toBe('asc');
  });

  it('falls back to defaults for unusable stored values', () => {
    localStorage.setItem(SORT_KEY, JSON.stringify({ timeline: 'sideways', collection: 'desc' }));
    const controls = new BrowseControlsStore();
    controls.enter('photos', 'timeline');
    expect(controls.sortDir).toBe('desc');
    controls.enter('album:1', 'collection');
    expect(controls.sortDir).toBe('desc');
  });

  it('resets only the active family', () => {
    const controls = new BrowseControlsStore();
    controls.enter('photos', 'timeline');
    controls.setSortDir('asc');
    controls.enter('album:1', 'collection');
    controls.setSortDir('desc');
    controls.reset();
    expect(controls.sortDir).toBe('asc');
    controls.enter('photos', 'timeline');
    expect(controls.sortDir).toBe('asc');
  });

  it('clears filters on a new context but keeps the family direction', () => {
    const controls = new BrowseControlsStore();
    controls.enter('album:1', 'collection');
    controls.setSortDir('desc');
    controls.favoriteOnly = true;
    controls.enter('album:2', 'collection');
    expect(controls.favoriteOnly).toBe(false);
    expect(controls.sortDir).toBe('desc');
  });

  it('leaves the family untouched for search', () => {
    const controls = new BrowseControlsStore();
    controls.enter('photos', 'timeline');
    controls.enter('search:cat', null);
    expect(controls.sortFamily).toBe('timeline');
  });
});

describe('browseControls filter', () => {
  beforeEach(() => {
    immichCapabilities.structuredSearch = false;
    browseControls.reset();
  });

  it('merges the active controls into the route scope', () => {
    browseControls.favoriteOnly = true;
    browseControls.rating = 'unrated';
    browseControls.filename = 'DSC';
    browseControls.visibility = 'archive';
    browseControls.takenAfter = '2024-01-01';

    expect(browseControls.filter({ scope: { tag: 't1' } })).toEqual({
      scope: { tag: 't1' },
      favorite: true,
      visibility: 'archive',
      imagesOnly: true,
      rating: 'unrated',
      filename: 'DSC',
      takenAfter: new Date('2024-01-01').toISOString()
    });

    browseControls.rating = 3;
    expect(browseControls.filter({}).rating).toEqual({ eq: 3 });
  });

  it.each([
    [true, false, true],
    [true, true, false],
    [false, false, false]
  ] as const)(
    'on structured search %s with flat-only %s sends a minimum rating: %s',
    (structured, flatOnly, sent) => {
      immichCapabilities.structuredSearch = structured;
      const keyBefore = browseControls.serverFilterKey(flatOnly);
      browseControls.rating = '4+';
      expect(browseControls.filter({}, flatOnly).rating).toEqual(sent ? { gte: 4 } : undefined);
      expect(browseControls.serverFilterKey(flatOnly) !== keyBefore).toBe(sent);
    }
  );

  it.each([
    [null, true],
    [3, true],
    [4, false]
  ])('hides a photo rated %s below 4+: %s', (rating, hidden) => {
    browseControls.rating = '4+';
    expect(browseControls.hides(asset(false, rating))).toBe(hidden);
  });
});
