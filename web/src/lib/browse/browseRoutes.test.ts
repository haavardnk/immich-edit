import { describe, expect, it } from 'vitest';
import { matchBrowseRoute } from './browseRoutes';

function route(href: string) {
  return matchBrowseRoute(new URL(href, 'http://restore.invalid'));
}

describe('matchBrowseRoute', () => {
  it.each([
    ['/photos', 'timeline', {}],
    ['/favorites', 'timeline', { favorite: true }],
    ['/albums/a1', 'collection', { scope: { album: 'a1' } }],
    ['/people/p1', 'collection', { scope: { person: 'p1' } }],
    ['/tags/t1', 'collection', { scope: { tag: 't1' } }]
  ])('resolves %s to a search feed', (href, family, filter) => {
    const match = route(href);
    expect(match).not.toBeNull();
    expect(match?.family).toBe(family);
    expect(match && 'feed' in match && match.feed.baseFilter()).toEqual(filter);
  });

  it('decodes an id that was percent encoded in the return path', () => {
    const match = route('/tags/parent%2Fchild');
    expect(match && 'feed' in match && match.feed.baseFilter()).toEqual({
      scope: { tag: 'parent/child' }
    });
  });

  it('carries the query and mode into a search feed', () => {
    const match = route('/search?q=%20sunset%20&mode=description');
    expect(match?.family).toBeNull();
    expect(match && 'feed' in match && match.feed.flatOnly?.()).toBe(true);
    expect(match && 'feed' in match && match.feed.baseFilter()).toEqual({});
  });

  it.each([
    ['/search', 'no query'],
    ['/folders', 'no folder'],
    ['/albums', 'no id'],
    ['/albums/a1/detail', 'nested path'],
    ['/settings', 'unknown route']
  ])('declines %s (%s)', (href) => {
    expect(route(href)).toBeNull();
  });

  it.each([
    ['/folders?path=%2Fmedia%2F2024', 'collection'],
    ['/edited', 'edited']
  ])('resolves %s to a direct loader', (href, family) => {
    const match = route(href);
    expect(match?.family).toBe(family);
    expect(match && 'load' in match).toBe(true);
  });
});
