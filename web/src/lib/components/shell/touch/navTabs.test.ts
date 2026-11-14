import { describe, expect, it } from 'vitest';
import { activeTab, crossesTabs, isTabRoot } from './navTabs';

describe('activeTab', () => {
  it.each([
    ['/photos', 'photos'],
    ['/albums', 'albums'],
    ['/albums/a1', 'albums'],
    ['/search', 'search'],
    ['/library', 'library'],
    ['/favorites', 'library'],
    ['/people', 'library'],
    ['/people/p1', 'library'],
    ['/tags', 'library'],
    ['/tags/t1', 'library'],
    ['/folders', 'library'],
    ['/edited', 'library']
  ])('marks %s as %s', (pathname, tab) => {
    expect(activeTab(pathname)).toBe(tab);
  });

  it.each(['/settings', '/settings/diagnostics', '/photosets', '/albumsx', '/'])(
    'leaves %s without a tab',
    (pathname) => {
      expect(activeTab(pathname)).toBeNull();
    }
  );
});

describe('isTabRoot', () => {
  it.each([
    ['/photos', true],
    ['/albums', true],
    ['/search', true],
    ['/library', true],
    ['/albums/a1', false],
    ['/people', false],
    ['/folders', false],
    ['/settings', false]
  ])('treats %s as a tab root: %s', (pathname, root) => {
    expect(isTabRoot(pathname)).toBe(root);
  });
});

describe('crossesTabs', () => {
  it('is true only between two different tabs', () => {
    expect(crossesTabs('/photos', '/albums/a1')).toBe(true);
    expect(crossesTabs('/albums', '/albums/a1')).toBe(false);
    expect(crossesTabs('/library', '/favorites')).toBe(false);
    expect(crossesTabs('/settings', '/albums/a1')).toBe(false);
    expect(crossesTabs('/albums/a1', '/settings')).toBe(false);
  });
});
