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

describe('browseControls rejected', () => {
  beforeEach(() => {
    immichCapabilities.structuredSearch = false;
    browseControls.reset();
  });

  it('shows rejected photos by default', () => {
    expect(browseControls.rejected).toBe('any');
    expect(browseControls.isDefault).toBe(true);
    expect(browseControls.isFiltered).toBe(false);
    expect(browseControls.hides(asset(true))).toBe(false);
  });

  it.each([
    ['hide', false, undefined],
    ['hide', true, 'hide'],
    ['only', false, 'only'],
    ['only', true, 'only']
  ] as const)('sends %s on structured search %s as %s', (rejected, structured, sent) => {
    immichCapabilities.structuredSearch = structured;
    const keyBefore = browseControls.serverFilterKey();
    browseControls.rejected = rejected;
    expect(browseControls.isFiltered).toBe(true);
    expect(browseControls.filter({}).rejected).toBe(sent);
    expect(browseControls.serverFilterKey() !== keyBefore).toBe(sent !== undefined);
    expect(browseControls.hides(asset(true))).toBe(rejected === 'hide');
    expect(browseControls.hides(asset(false))).toBe(rejected === 'only');
  });

  it('is cleared by reset', () => {
    browseControls.rejected = 'only';
    browseControls.reset();
    expect(browseControls.rejected).toBe('any');
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
    localStorage.setItem(
      SORT_KEY,
      JSON.stringify({
        timeline: 'sideways',
        collection: 'desc',
        edited: { field: 'uploaded', direction: 'desc' }
      })
    );
    const controls = new BrowseControlsStore();
    controls.enter('photos', 'timeline');
    expect(controls.sort).toEqual({ field: 'taken', direction: 'desc' });
    controls.enter('album:1', 'collection');
    expect(controls.sort).toEqual({ field: 'taken', direction: 'desc' });
    controls.enter('edited', 'edited');
    expect(controls.sort).toEqual({ field: 'taken', direction: 'desc' });
  });

  it('remembers the date field per family and resets it', () => {
    const controls = new BrowseControlsStore();
    controls.enter('photos', 'timeline');
    const keyBefore = controls.serverFilterKey();
    controls.setSortField('added');
    expect(controls.serverFilterKey()).not.toBe(keyBefore);
    expect(controls.isDefault).toBe(false);
    controls.setSortDir('asc');
    controls.enter('album:1', 'collection');
    expect(controls.sortField).toBe('taken');

    const reloaded = new BrowseControlsStore();
    reloaded.enter('photos', 'timeline');
    expect(reloaded.sort).toEqual({ field: 'added', direction: 'asc' });
    reloaded.reset();
    expect(reloaded.sort).toEqual({ field: 'taken', direction: 'desc' });
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
    'on structured search %s with flat-only %s sends exclusions: %s',
    (structured, flatOnly, sent) => {
      immichCapabilities.structuredSearch = structured;
      const keyBefore = browseControls.serverFilterKey(flatOnly);
      browseControls.rejected = 'hide';
      browseControls.label = 'none';
      browseControls.rating = '4+';
      const { rejected, label, rating } = browseControls.filter({}, flatOnly);
      expect([rejected, label, rating]).toEqual(
        sent ? ['hide', 'none', { gte: 4 }] : [undefined, undefined, undefined]
      );
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

  it.each([
    ['red', false, 'red'],
    ['none', false, undefined],
    ['none', true, 'none']
  ] as const)('sends label %s on structured search %s as %s', (label, structured, sent) => {
    immichCapabilities.structuredSearch = structured;
    const keyBefore = browseControls.serverFilterKey();
    browseControls.label = label;
    expect(browseControls.filter({}).label).toBe(sent);
    expect(browseControls.serverFilterKey() !== keyBefore).toBe(sent !== undefined);
  });
});
