import { describe, expect, it } from 'vitest';
import { matchBrowseRoute } from './browseRestore';

function route(href: string) {
  return matchBrowseRoute(new URL(href, 'http://restore.invalid'));
}

describe('matchBrowseRoute', () => {
  it.each([
    ['/photos', 'photos', 'timeline', {}],
    ['/favorites', 'favorites', 'timeline', { isFavorite: true }],
    ['/albums/a1', 'album:a1', 'collection', { albumIds: ['a1'] }],
    ['/people/p1', 'person:p1', 'collection', { personIds: ['p1'] }],
    ['/tags/t1', 'tag:t1', 'collection', { tagIds: ['t1'] }]
  ])('resolves %s to a search feed', (href, context, family, body) => {
    const match = route(href);
    expect(match).not.toBeNull();
    expect(match?.context).toBe(context);
    expect(match?.family).toBe(family);
    expect(match && 'feed' in match && match.feed.baseBody()).toEqual(body);
  });

  it('decodes an id that was percent encoded in the return path', () => {
    const match = route('/tags/parent%2Fchild');
    expect(match?.context).toBe('tag:parent/child');
  });

  it('carries the query and mode into a search feed', () => {
    const match = route('/search?q=%20sunset%20&mode=filename');
    expect(match?.context).toBe('search:sunset');
    expect(match && 'feed' in match && match.feed.baseBody()).toEqual({});
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
    ['/folders?path=%2Fmedia%2F2024', 'folders', 'collection'],
    ['/edited', 'edited', 'edited']
  ])('resolves %s to a direct loader', (href, context, family) => {
    const match = route(href);
    expect(match?.context).toBe(context);
    expect(match?.family).toBe(family);
    expect(match && 'load' in match).toBe(true);
  });
});
