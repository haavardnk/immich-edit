import { afterEach, describe, expect, it } from 'vitest';
import { FILTER_DEFAULTS, type BrowseFilters } from '$lib/stores/browseControls.svelte';
import { immichCapabilities } from '$lib/stores/immichCapabilities.svelte';
import { isTypedEdit, readFilterParams, withFilterParams } from './filterParams';

const FULL: BrowseFilters = {
  favoriteOnly: true,
  rating: '3+',
  filename: 'IMG 0001+',
  visibility: 'archive',
  takenAfter: '2024-01-01',
  takenBefore: '2024-02-29',
  rejected: 'only',
  label: 'purple',
  make: ['SONY', 'Canon'],
  model: ['ILCE-7M3'],
  lensModel: ['FE 35mm F1.8'],
  country: ['Norway'],
  state: ['Vestland'],
  city: ['Bergen']
};

function read(search: string): BrowseFilters {
  return readFilterParams(new URLSearchParams(search));
}

afterEach(() => {
  immichCapabilities.structuredSearch = false;
});

describe('filter params', () => {
  it('round-trips every filter and keeps unrelated params', () => {
    immichCapabilities.structuredSearch = true;
    const url = withFilterParams(new URL('http://x.invalid/search?q=cat&mode=filename'), FULL);
    expect(url.searchParams.get('q')).toBe('cat');
    expect(url.searchParams.get('mode')).toBe('filename');
    expect(readFilterParams(url.searchParams)).toEqual(FULL);
  });

  it('writes nothing for defaults and replaces stale filter params', () => {
    const url = withFilterParams(
      new URL('http://x.invalid/folders?path=%2Fa&favorite=1&make=SONY'),
      FILTER_DEFAULTS
    );
    expect(url.search).toBe('?path=%2Fa');
  });

  it('writes params in one canonical order', () => {
    const url = withFilterParams(new URL('http://x.invalid/photos?label=red&favorite=1'), {
      ...FILTER_DEFAULTS,
      favoriteOnly: true,
      label: 'red'
    });
    expect(url.search).toBe('?favorite=1&label=red');
  });

  it.each([
    ['rating=banana', 'rating', 'any'],
    ['rating=0', 'rating', 'any'],
    ['rating=4', 'rating', 4],
    ['rating=unrated', 'rating', 'unrated'],
    ['rating=3%2B', 'rating', 'any'],
    ['visibility=trash', 'visibility', 'timeline'],
    ['rejected=maybe', 'rejected', 'any'],
    ['label=pink', 'label', 'any'],
    ['favorite=true', 'favoriteOnly', false],
    ['after=2024-02-30', 'takenAfter', ''],
    ['after=yesterday', 'takenAfter', ''],
    ['before=2024-1-5', 'takenBefore', ''],
    ['before=2024-12-31', 'takenBefore', '2024-12-31'],
    ['make=SONY&make=Canon', 'make', []],
    ['lens=&lens=FE', 'lensModel', ['FE']],
    ['lens=', 'lensModel', ['']]
  ] as [string, keyof BrowseFilters, unknown][])(
    'reads %s as %s %j without structured search',
    (search, key, expected) => {
      expect(read(search)[key]).toEqual(expected);
    }
  );

  it.each([
    ['rating=3%2B', 'rating', '3+'],
    ['rating=2-', 'rating', '2-'],
    ['make=SONY&make=Canon&make=SONY', 'make', ['SONY', 'Canon']]
  ] as [string, keyof BrowseFilters, unknown][])(
    'reads %s as %s %j with structured search',
    (search, key, expected) => {
      immichCapabilities.structuredSearch = true;
      expect(read(search)[key]).toEqual(expected);
    }
  );

  it.each([
    [{ filename: 'IMG' }, { filename: 'IMG_1' }, true],
    [{ takenAfter: '2024-01-01' }, { takenAfter: '2024-01-02' }, true],
    [{}, { filename: 'I' }, false],
    [{ filename: 'I' }, {}, false],
    [{ filename: 'IMG' }, { filename: 'IMG_1', favoriteOnly: true }, false],
    [{ make: ['SONY'] }, { make: ['Canon'] }, false],
    [{}, {}, false]
  ] as [Partial<BrowseFilters>, Partial<BrowseFilters>, boolean][])(
    'treats %j -> %j as a typed edit: %s',
    (from, to, expected) => {
      expect(isTypedEdit({ ...FILTER_DEFAULTS, ...from }, { ...FILTER_DEFAULTS, ...to })).toBe(
        expected
      );
    }
  );
});
