import { describe, expect, it } from 'vitest';
import { pageDirection, pageMotion } from './pageDirection.ts';

const TABS = new Set(['/photos', '/albums', '/search', '/library']);
const SIDEBAR = new Set([
  ...TABS,
  '/favorites',
  '/people',
  '/tags',
  '/folders',
  '/edited',
  '/settings'
]);

function url(path: string): URL {
  return new URL(path, 'https://app.test');
}

function direction(from: string, to: string, roots: ReadonlySet<string> = TABS): string {
  return pageDirection(url(from), url(to), roots);
}

describe('pageDirection', () => {
  it.each([
    ['/photos', '/albums'],
    ['/library', '/search'],
    ['/people/a', '/tags/b']
  ])('cross-fades between %s and %s at the same depth', (from, to) => {
    expect(direction(from, to)).toBe('tab');
  });

  it.each([
    ['/library', '/people'],
    ['/people', '/people/a'],
    ['/library', '/people/a'],
    ['/albums', '/albums/a'],
    ['/photos', '/settings'],
    ['/library', '/folders?path=a/b']
  ])('pushes from %s into %s', (from, to) => {
    expect(direction(from, to)).toBe('forward');
    expect(direction(to, from)).toBe('back');
  });

  it('treats sidebar destinations as roots outside the compact layout', () => {
    expect(direction('/photos', '/people', SIDEBAR)).toBe('tab');
    expect(direction('/people', '/people/a', SIDEBAR)).toBe('forward');
    expect(direction('/settings/diagnostics', '/settings', SIDEBAR)).toBe('back');
  });

  it.each([
    ['/photos', '/photos?favorite=true'],
    ['/folders?path=a', '/folders?path=a/b'],
    ['/photos', '/assets/a'],
    ['/assets/a', '/albums/b']
  ])('keeps %s to %s still', (from, to) => {
    expect(direction(from, to)).toBe('none');
  });
});

describe('pageMotion', () => {
  it.each([
    ['none', true, null],
    ['tab', true, 'fade'],
    ['tab', false, 'fade'],
    ['forward', true, 'push'],
    ['back', true, 'pop'],
    ['forward', false, 'rise'],
    ['back', false, 'sink']
  ] as const)('maps %s on compact=%s to %s', (from, compact, motion) => {
    expect(pageMotion(from, compact)).toBe(motion);
  });
});
