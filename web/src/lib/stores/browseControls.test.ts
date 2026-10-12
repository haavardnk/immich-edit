import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
  BrowseControlsStore,
  browseControls,
  FILTER_DEFAULTS,
  type SortFamily
} from './browseControls.svelte';
import { immichCapabilities } from './immichCapabilities.svelte';
import { hides } from '$lib/browse/loadedFilter';
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
    expect(browseControls.filters.rejected).toBe('any');
    expect(browseControls.isDefault).toBe(true);
    expect(browseControls.isFiltered).toBe(false);
    expect(hides(browseControls.filters, asset(true))).toBe(false);
  });

  it.each([
    ['hide', false, undefined],
    ['hide', true, 'hide'],
    ['only', false, 'only'],
    ['only', true, 'only']
  ] as const)('sends %s on structured search %s as %s', (rejected, structured, sent) => {
    immichCapabilities.structuredSearch = structured;
    const keyBefore = browseControls.serverFilterKey();
    browseControls.filters.rejected = rejected;
    expect(browseControls.isFiltered).toBe(true);
    expect(browseControls.filter({}).rejected).toBe(sent);
    expect(browseControls.serverFilterKey() !== keyBefore).toBe(sent !== undefined);
    expect(hides(browseControls.filters, asset(true))).toBe(rejected === 'hide');
    expect(hides(browseControls.filters, asset(false))).toBe(rejected === 'only');
  });

  it('is cleared by reset', () => {
    browseControls.filters.rejected = 'only';
    browseControls.reset();
    expect(browseControls.filters.rejected).toBe('any');
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
    controls.enter(family, FILTER_DEFAULTS);
    expect(controls.sortDir).toBe(dir);
    expect(controls.isDefault).toBe(true);
  });

  it('keeps each family independent and survives a reload', () => {
    const controls = new BrowseControlsStore();
    controls.enter('timeline', FILTER_DEFAULTS);
    controls.setSortDir('asc');
    controls.enter('collection', FILTER_DEFAULTS);
    expect(controls.sortDir).toBe('asc');
    controls.setSortDir('desc');
    controls.enter('timeline', FILTER_DEFAULTS);
    expect(controls.sortDir).toBe('asc');

    const reloaded = new BrowseControlsStore();
    reloaded.enter('collection', FILTER_DEFAULTS);
    expect(reloaded.sortDir).toBe('desc');
    reloaded.enter('edited', FILTER_DEFAULTS);
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
    controls.enter('timeline', FILTER_DEFAULTS);
    expect(controls.sort).toEqual({ field: 'taken', direction: 'desc' });
    controls.enter('collection', FILTER_DEFAULTS);
    expect(controls.sort).toEqual({ field: 'taken', direction: 'desc' });
    controls.enter('edited', FILTER_DEFAULTS);
    expect(controls.sort).toEqual({ field: 'taken', direction: 'desc' });
  });

  it('remembers the date field per family and resets it', () => {
    const controls = new BrowseControlsStore();
    controls.enter('timeline', FILTER_DEFAULTS);
    const keyBefore = controls.serverFilterKey();
    controls.setSortField('added');
    expect(controls.serverFilterKey()).not.toBe(keyBefore);
    expect(controls.isDefault).toBe(false);
    controls.setSortDir('asc');
    controls.enter('collection', FILTER_DEFAULTS);
    expect(controls.sortField).toBe('taken');

    const reloaded = new BrowseControlsStore();
    reloaded.enter('timeline', FILTER_DEFAULTS);
    expect(reloaded.sort).toEqual({ field: 'added', direction: 'asc' });
    reloaded.reset();
    expect(reloaded.sort).toEqual({ field: 'taken', direction: 'desc' });
  });

  it.each(['rating', 'size'] as const)(
    'recalls a stored %s sort only where structured search exists',
    (field) => {
      localStorage.setItem(SORT_KEY, JSON.stringify({ collection: { field, direction: 'desc' } }));
      const controls = new BrowseControlsStore();
      controls.enter('collection', FILTER_DEFAULTS);
      immichCapabilities.structuredSearch = false;
      expect(controls.sort).toEqual({ field: 'taken', direction: 'asc' });
      expect(controls.isDefault).toBe(true);
      immichCapabilities.structuredSearch = true;
      expect(controls.sort).toEqual({ field, direction: 'desc' });
      expect(controls.isDefault).toBe(false);
    }
  );

  it('resets only the active family', () => {
    const controls = new BrowseControlsStore();
    controls.enter('timeline', FILTER_DEFAULTS);
    controls.setSortDir('asc');
    controls.enter('collection', FILTER_DEFAULTS);
    controls.setSortDir('desc');
    controls.reset();
    expect(controls.sortDir).toBe('asc');
    controls.enter('timeline', FILTER_DEFAULTS);
    expect(controls.sortDir).toBe('asc');
  });

  it('takes the filters it enters with but keeps the family direction', () => {
    const controls = new BrowseControlsStore();
    controls.enter('collection', FILTER_DEFAULTS);
    controls.setSortDir('desc');
    controls.filters.favoriteOnly = true;
    controls.enter('collection', { ...FILTER_DEFAULTS, rating: 3 });
    expect(controls.filters.favoriteOnly).toBe(false);
    expect(controls.filters.rating).toBe(3);
    expect(controls.sortDir).toBe('desc');
  });

  it('flags a prune that drops values, once', () => {
    const controls = new BrowseControlsStore();
    controls.filters.make = ['SONY', 'Canon'];
    controls.prune('make', ['SONY', 'Canon', 'Nikon']);
    expect(controls.takeCorrected()).toBe(false);
    controls.prune('make', ['SONY']);
    expect(controls.filters.make).toEqual(['SONY']);
    expect(controls.takeCorrected()).toBe(true);
    expect(controls.takeCorrected()).toBe(false);
  });

  it('leaves the family untouched for search', () => {
    const controls = new BrowseControlsStore();
    controls.enter('timeline', FILTER_DEFAULTS);
    controls.enter(null, FILTER_DEFAULTS);
    expect(controls.sortFamily).toBe('timeline');
  });
});

describe('browseControls filter', () => {
  beforeEach(() => {
    immichCapabilities.structuredSearch = false;
    browseControls.reset();
  });

  it('merges the active controls into the route scope', () => {
    browseControls.filters.favoriteOnly = true;
    browseControls.filters.notInAlbum = true;
    browseControls.filters.untagged = true;
    browseControls.filters.noPeople = true;
    browseControls.filters.rating = 'unrated';
    browseControls.filters.filename = 'DSC';
    browseControls.filters.visibility = 'archive';
    browseControls.filters.takenAfter = '2024-01-01';

    expect(browseControls.filter({ scope: { tag: 't1' } })).toEqual({
      scope: { tag: 't1' },
      favorite: true,
      notInAlbum: true,
      untagged: true,
      noPeople: true,
      visibility: 'archive',
      imagesOnly: true,
      rating: 'unrated',
      filename: 'DSC',
      takenAfter: new Date('2024-01-01').toISOString()
    });

    browseControls.filters.rating = 3;
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
      browseControls.filters.rejected = 'hide';
      browseControls.filters.label = 'none';
      browseControls.filters.rating = '4+';
      const { rejected, label, rating } = browseControls.filter({}, flatOnly);
      expect([rejected, label, rating]).toEqual(
        sent ? ['hide', 'none', { gte: 4 }] : [undefined, undefined, undefined]
      );
      expect(browseControls.serverFilterKey(flatOnly) !== keyBefore).toBe(sent);
    }
  );

  it.each([
    ['4+', null, true],
    ['4+', 3, true],
    ['4+', 4, false],
    ['2-', null, false],
    ['2-', 2, false],
    ['2-', 3, true],
    [2, 3, false]
  ] as const)('filter %s hides a photo rated %s: %s', (filter, rating, hidden) => {
    browseControls.filters.rating = filter;
    expect(hides(browseControls.filters, asset(false, rating))).toBe(hidden);
  });

  it('sends a maximum rating on structured search', () => {
    immichCapabilities.structuredSearch = true;
    browseControls.filters.rating = '2-';
    expect(browseControls.filter({}).rating).toEqual({ lte: 2 });
  });

  it.each([
    ['red', false, 'red'],
    ['none', false, undefined],
    ['none', true, 'none']
  ] as const)('sends label %s on structured search %s as %s', (label, structured, sent) => {
    immichCapabilities.structuredSearch = structured;
    const keyBefore = browseControls.serverFilterKey();
    browseControls.filters.label = label;
    expect(browseControls.filter({}).label).toBe(sent);
    expect(browseControls.serverFilterKey() !== keyBefore).toBe(sent !== undefined);
  });

  it.each([
    [false, false, ['SONY'], ['SONY']],
    [false, false, ['SONY', 'FUJIFILM'], undefined],
    [true, false, ['SONY', 'FUJIFILM'], ['SONY', 'FUJIFILM']],
    [true, true, ['SONY', 'FUJIFILM'], undefined],
    [false, false, [''], [null]]
  ])(
    'on structured search %s with flat-only %s sends makes %j as %j',
    (structured, flatOnly, makes, sent) => {
      immichCapabilities.structuredSearch = structured;
      const keyBefore = browseControls.serverFilterKey(flatOnly);
      browseControls.filters.make = makes;
      expect(browseControls.isFiltered).toBe(true);
      expect(browseControls.filter({}, flatOnly).make).toEqual(sent);
      expect(browseControls.serverFilterKey(flatOnly) !== keyBefore).toBe(sent !== undefined);
    }
  );

  it.each([
    [{ make: 'SONY', lensModel: 'FE 35mm F1.8', city: 'Bergen' }, false],
    [{ make: 'FUJIFILM', lensModel: 'FE 35mm F1.8', city: 'Oslo' }, false],
    [{ make: 'SONY', lensModel: 'FE 85mm F1.8', city: 'Bergen' }, true],
    [{ make: 'SONY', lensModel: 'FE 35mm F1.8', city: 'Tromsø' }, true],
    [null, true]
  ])('camera and place filters hide %j: %s', (exif, hidden) => {
    browseControls.filters.make = ['SONY', 'FUJIFILM'];
    browseControls.filters.lensModel = ['FE 35mm F1.8'];
    browseControls.filters.city = ['Bergen', 'Oslo'];
    const photo = { ...asset(false), exifInfo: exif as ExifInfo | null };
    expect(hides(browseControls.filters, photo)).toBe(hidden);
  });

  it.each([
    [{ lensModel: 'FE 35mm F1.8' }, true],
    [{ lensModel: null }, false],
    [null, false]
  ])('an unknown lens filter hides %j: %s', (exif, hidden) => {
    browseControls.filters.lensModel = [''];
    const photo = { ...asset(false), exifInfo: exif as ExifInfo | null };
    expect(hides(browseControls.filters, photo)).toBe(hidden);
  });

  it('sends the size range in bytes and refetches on change', () => {
    const keyBefore = browseControls.serverFilterKey();
    browseControls.filters.minSize = 2.5;
    browseControls.filters.maxSize = 100;
    const { minSize, maxSize } = browseControls.filter({});
    expect({ minSize, maxSize }).toEqual({ minSize: 2_500_000, maxSize: 100_000_000 });
    expect(browseControls.serverFilterKey()).not.toBe(keyBefore);
  });
});
