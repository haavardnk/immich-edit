import { afterEach, describe, expect, it } from 'vitest';
import { FILTER_DEFAULTS, type BrowseFilters } from '$lib/stores/browseControls.svelte';
import { immichCapabilities } from '$lib/stores/immichCapabilities.svelte';
import { isTypedEdit, readFilterParams, withFilterParams } from './filterParams';

const FULL: BrowseFilters = {
  favoriteOnly: true,
  notInAlbum: true,
  untagged: true,
  noPeople: true,
  albums: { ids: ['a1'], match: 'all' },
  people: { ids: ['p1', 'p2'], match: 'none' },
  tags: { ids: ['t1'], match: 'any' },
  rating: '3+',
  filename: 'IMG 0001+',
  minSize: 0.5,
  maxSize: 25,
  visibility: 'archive',
  takenAfter: '2024-01-01',
  takenBefore: '2024-02-29',
  addedAfter: '2025-03-01',
  addedBefore: '2025-03-31',
  rejected: 'only',
  label: 'purple',
  make: ['SONY', 'Canon'],
  model: ['ILCE-7M3'],
  lensModel: ['FE 35mm F1.8'],
  country: ['Norway'],
  state: ['Vestland'],
  city: ['Bergen']
};

function read(search: string, pathname = '/photos'): BrowseFilters {
  return readFilterParams(new URL(`http://x.invalid${pathname}?${search}`));
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
    expect(readFilterParams(url)).toEqual(FULL);
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
    ['addedafter=2025-02-30', 'addedAfter', ''],
    ['addedbefore=2025-03-31', 'addedBefore', '2025-03-31'],
    ['make=SONY&make=Canon', 'make', []],
    ['lens=&lens=FE', 'lensModel', ['FE']],
    ['lens=', 'lensModel', ['']],
    ['minsize=25', 'minSize', null]
  ] as [string, keyof BrowseFilters, unknown][])(
    'reads %s as %s %j without structured search',
    (search, key, expected) => {
      expect(read(search)[key]).toEqual(expected);
    }
  );

  it.each([
    ['rating=3%2B', 'rating', '3+'],
    ['rating=2-', 'rating', '2-'],
    ['make=SONY&make=Canon&make=SONY', 'make', ['SONY', 'Canon']],
    ['minsize=2.5', 'minSize', 2.5],
    ['maxsize=0', 'maxSize', null],
    ['maxsize=-3', 'maxSize', null],
    ['maxsize=big', 'maxSize', null]
  ] as [string, keyof BrowseFilters, unknown][])(
    'reads %s as %s %j with structured search',
    (search, key, expected) => {
      immichCapabilities.structuredSearch = true;
      expect(read(search)[key]).toEqual(expected);
    }
  );

  it.each([
    [true, 'person=p1&person=p2&person=p1&peoplematch=any', { ids: ['p1', 'p2'], match: 'any' }],
    [true, 'person=p1&peoplematch=bogus', { ids: ['p1'], match: 'all' }],
    [true, 'peoplematch=none', { ids: [], match: 'all' }],
    [false, 'person=p1&person=p2', { ids: ['p1', 'p2'], match: 'all' }],
    [false, 'person=p1&peoplematch=none', { ids: [], match: 'all' }]
  ])('reads people with structured search %s from %s', (structured, search, expected) => {
    immichCapabilities.structuredSearch = structured;
    expect(read(search).people).toEqual(expected);
  });

  it('writes the people match only when it is not all', () => {
    const people = (match: 'all' | 'any') =>
      withFilterParams(new URL('http://x.invalid/photos'), {
        ...FILTER_DEFAULTS,
        people: { ids: ['p1'], match }
      }).search;
    expect(people('all')).toBe('?person=p1');
    expect(people('any')).toBe('?person=p1&peoplematch=any');
  });

  it.each([
    ['/photos', true],
    ['/people/p1', true],
    ['/search', true],
    ['/albums/a1', false],
    ['/folders', false],
    ['/edited', false]
  ])('reads noalbum on %s: %s', (pathname, expected) => {
    expect(read('noalbum=1&favorite=1', pathname)).toMatchObject({
      notInAlbum: expected,
      favoriteOnly: true
    });
  });

  it.each([
    ['/photos', true, true, true, 3],
    ['/photos', false, true, false, null],
    ['/tags/t1', true, false, true, 3],
    ['/people/p1', true, true, false, 3],
    ['/albums/a1', true, true, true, 3],
    ['/search?q=cat&mode=filename', true, true, true, 3],
    ['/search?q=cat&mode=description', true, true, false, null],
    ['/folders?path=%2Fa', true, false, false, 3],
    ['/edited', false, false, false, 3]
  ])(
    'reads untagged, nopeople and minsize on %s with structured search %s: %s, %s, %s',
    (route, structured, untagged, noPeople, minSize) => {
      immichCapabilities.structuredSearch = structured;
      const url = new URL(`http://x.invalid${route}`);
      url.searchParams.set('untagged', '1');
      url.searchParams.set('nopeople', '1');
      url.searchParams.set('minsize', '3');
      expect(readFilterParams(url)).toMatchObject({ untagged, noPeople, minSize });
    }
  );

  it.each([
    ['/photos', '3+'],
    ['/folders?path=%2Fa', '3+'],
    ['/search?q=cat&mode=filename', '3+'],
    ['/search?q=cat&mode=description', 'any']
  ])('reads rating=3+ on %s with structured search as %s', (route, expected) => {
    immichCapabilities.structuredSearch = true;
    const url = new URL(`http://x.invalid${route}`);
    url.searchParams.set('rating', '3+');
    expect(readFilterParams(url).rating).toBe(expected);
  });

  it.each([
    ['/photos', 'archive'],
    ['/folders?path=%2Fa', 'timeline'],
    ['/edited', 'timeline']
  ])('reads visibility=archive on %s as %s', (route, expected) => {
    const url = new URL(`http://x.invalid${route}`);
    url.searchParams.set('visibility', 'archive');
    expect(readFilterParams(url).visibility).toBe(expected);
  });

  it.each([
    [{ filename: 'IMG' }, { filename: 'IMG_1' }, true],
    [{ takenAfter: '2024-01-01' }, { takenAfter: '2024-01-02' }, true],
    [{ minSize: 2 }, { minSize: 25 }, true],
    [{}, { maxSize: 2 }, false],
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
