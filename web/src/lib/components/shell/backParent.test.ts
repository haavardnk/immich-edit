import { describe, expect, it } from 'vitest';
import { backParent, backSkips } from './backParent';

describe('backParent', () => {
  it.each([
    ['/favorites', '/library'],
    ['/map', '/library'],
    ['/edited', '/library'],
    ['/people', '/library'],
    ['/tags', '/library'],
    ['/folders', '/library'],
    ['/folders?path=%2Fphotos', '/folders'],
    ['/folders?path=%2Fphotos%2F2026', '/folders?path=%2Fphotos'],
    ['/folders?path=%2Fphotos%2F2026%2FNorway', '/folders?path=%2Fphotos%2F2026'],
    ['/people/p1', '/people'],
    ['/tags/t1', '/tags'],
    ['/albums/a1', '/albums'],
    ['/settings/diagnostics', '/settings'],
    ['/settings', '/photos'],
    ['/somewhere', '/photos']
  ])('goes back from %s to %s', (from, parent) => {
    expect(backParent(new URL(from, 'http://localhost'))).toBe(parent);
  });

  it('leaves the map for its parent instead of an earlier map view', () => {
    expect(backSkips('/map', '/map')).toBe(true);
    expect(backSkips('/map', '/library')).toBe(false);
    expect(backSkips('/albums/a1', '/albums/a1')).toBe(false);
  });
});
