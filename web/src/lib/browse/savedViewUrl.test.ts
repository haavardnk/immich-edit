import { describe, expect, it } from 'vitest';
import {
  SAVED_VIEW_URL_MAX,
  isSavedViewUrl,
  parseSavedViewUrl,
  savedViewPath,
  savedViewUrl
} from './savedViewUrl';

const NEWEST = { field: 'taken', direction: 'desc' } as const;

describe('savedViewUrl', () => {
  it('appends sort and size to the path and its filters', () => {
    expect(savedViewUrl('/photos?model=X-T5&rating=3', NEWEST, 'lg')).toBe(
      '/photos?model=X-T5&rating=3&sort=taken-desc&size=lg'
    );
  });

  it('omits the sort where the page has none', () => {
    expect(savedViewUrl('/search?q=fjord', null, 'md')).toBe('/search?q=fjord&size=md');
  });

  it('replaces sort and size already in the path', () => {
    expect(savedViewUrl('/photos?size=sm&sort=added-asc', NEWEST, 'xl')).toBe(
      '/photos?sort=taken-desc&size=xl'
    );
  });

  it('encodes values to printable ascii and drops the fragment', () => {
    const url = savedViewUrl('/folders?path=/2026/Nordic trip#x', NEWEST, 'md');
    expect(url).toBe('/folders?path=%2F2026%2FNordic+trip&sort=taken-desc&size=md');
    expect(url).toMatch(/^\/[!-~]*$/);
  });

  it('encodes non-ascii filter values and backslashes', () => {
    expect(savedViewUrl('/photos?city=Tromsø&filename=a\\b', NEWEST, 'md')).toBe(
      '/photos?city=Troms%C3%B8&filename=a%5Cb&sort=taken-desc&size=md'
    );
  });

  it('refuses a url the server would not store', () => {
    const tags = Array.from({ length: 60 }, (_, i) => `tag=${'0'.repeat(36)}${i}`).join('&');
    expect(savedViewUrl(`/photos?${tags}`, NEWEST, 'md')).toBeNull();
  });
});

describe('isSavedViewUrl', () => {
  it.each([
    ['/photos', true],
    ['/photos?city=Troms%C3%B8', true],
    ['', false],
    ['photos', false],
    ['//elsewhere.example/photos', false],
    ['/\\elsewhere.example', false],
    ['https://elsewhere.example/photos', false],
    ['/photos#top', false],
    ['/photos?a=b c', false],
    ['/photos?city=Tromsø', false],
    [`/${'a'.repeat(SAVED_VIEW_URL_MAX - 1)}`, true],
    [`/${'a'.repeat(SAVED_VIEW_URL_MAX)}`, false],
    ['/.//evil.com', false],
    ['/a/..//evil.com', false],
    ['/%2e//evil.com', false],
    ['/%2E%2E//evil.com', false],
    ['/a/%2e%2e//evil.com?size=sm', false],
    ['/./%5Cevil.com', true],
    ['/photos/../albums', true]
  ])('judges %j as %s', (url, valid) => {
    expect(isSavedViewUrl(url)).toBe(valid);
  });
});

describe('parseSavedViewUrl', () => {
  it('splits the path from sort and size', () => {
    expect(parseSavedViewUrl('/photos?model=X-T5&sort=rating-asc&size=xl')).toEqual({
      path: '/photos?model=X-T5',
      sort: { field: 'rating', direction: 'asc' },
      size: 'xl'
    });
  });

  it('drops unknown sort fields, directions and sizes', () => {
    for (const sort of ['name-asc', 'taken-up', 'taken', 'taken-desc-x', 'constructor-asc']) {
      expect(parseSavedViewUrl(`/photos?sort=${sort}&size=huge`)).toEqual({ path: '/photos' });
    }
  });

  it('rejects urls that leave the site', () => {
    expect(parseSavedViewUrl('//elsewhere.example/photos?size=sm')).toBeNull();
    expect(parseSavedViewUrl('https://elsewhere.example/photos')).toBeNull();
  });

  it.each(['/.//evil.com', '/a/..//evil.com', '/%2e//evil.com', '/%2e%2e//evil.com?size=sm'])(
    'rejects %j, whose dot segments collapse into another host',
    (url) => {
      expect(parseSavedViewUrl(url)).toBeNull();
    }
  );

  it('keeps an encoded backslash inside the path', () => {
    expect(parseSavedViewUrl('/./%5Cevil.com')).toEqual({ path: '/%5Cevil.com' });
  });

  it('round-trips what it builds', () => {
    const built = savedViewUrl('/albums/a1?label=red', { field: 'added', direction: 'asc' }, 'sm');
    expect(parseSavedViewUrl(built ?? '')).toEqual({
      path: '/albums/a1?label=red',
      sort: { field: 'added', direction: 'asc' },
      size: 'sm'
    });
  });
});

describe('savedViewPath', () => {
  it('matches the same page whatever its sort, size or encoding', () => {
    expect(savedViewPath('/folders?path=%2Fa+b&sort=taken-asc&size=lg')).toBe(
      savedViewPath('/folders?path=/a%20b')
    );
  });

  it('matches filters written in a different order', () => {
    expect(savedViewPath('/photos?country=Norway&favorite=1&size=sm')).toBe(
      savedViewPath('/photos?favorite=1&country=Norway')
    );
    expect(savedViewPath('/folders?model=X-T5&path=%2Fa')).toBe(
      savedViewPath('/folders?path=%2Fa&model=X-T5')
    );
  });

  it('tells different filters apart', () => {
    expect(savedViewPath('/photos?rating=3')).not.toBe(savedViewPath('/photos?rating=4'));
  });
});
