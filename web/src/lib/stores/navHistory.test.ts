import type { AfterNavigate } from '$app/navigation';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { hasInAppPrevious, navHistory, nextDepth, routeKey } from './navHistory';

const nav = vi.hoisted(() => ({ goto: vi.fn(() => Promise.resolve()) }));

vi.mock('$app/navigation', () => ({ goto: nav.goto }));

const browser = { length: 1, back: vi.fn() };

function step(
  type: AfterNavigate['type'],
  from: string | null,
  to: string,
  delta?: number
): AfterNavigate {
  const target = { url: new URL(to, 'http://localhost'), params: {}, route: { id: null } };
  const source = from
    ? { url: new URL(from, 'http://localhost'), params: {}, route: { id: null } }
    : null;
  return {
    type,
    from: source,
    to: target,
    delta,
    willUnload: false,
    complete: Promise.resolve()
  } as AfterNavigate;
}

function push(type: 'link' | 'goto' | 'form', from: string, to: string): void {
  browser.length += 1;
  navHistory.record(step(type, from, to));
}

function stubNavigation(index: number, urls: string[]): void {
  Object.defineProperty(globalThis, 'navigation', {
    value: {
      currentEntry: { index, url: `http://localhost${urls[index] ?? '/'}` },
      entries: () => urls.map((url, i) => ({ index: i, url: `http://localhost${url}` }))
    },
    configurable: true
  });
}

beforeEach(() => {
  browser.length = 1;
  Object.defineProperty(globalThis, 'history', { value: browser, configurable: true });
  navHistory.record(step('enter', null, '/photos'));
});

afterEach(() => {
  Reflect.deleteProperty(globalThis, 'navigation');
  Reflect.deleteProperty(globalThis, 'history');
  browser.back.mockClear();
  nav.goto.mockClear();
});

describe('nextDepth', () => {
  it.each([
    ['enter resets', 3, { type: 'enter', pushed: true }, 0],
    ['a link that pushed counts', 0, { type: 'link', pushed: true }, 1],
    ['a link that replaced does not count', 1, { type: 'link', pushed: false }, 1],
    ['goto that pushed counts', 1, { type: 'goto', pushed: true }, 2],
    ['goto that replaced keeps the depth', 1, { type: 'goto', pushed: false }, 1],
    ['a form that pushed counts', 0, { type: 'form', pushed: true }, 1],
    ['back pops', 2, { type: 'popstate', delta: -1, pushed: false }, 1],
    ['forward pushes again', 1, { type: 'popstate', delta: 1, pushed: false }, 2],
    ['back past the entry stays at zero', 0, { type: 'popstate', delta: -2, pushed: false }, 0]
  ] as const)('%s', (_, depth, next, expected) => {
    expect(nextDepth(depth, next)).toBe(expected);
  });
});

describe('hasInAppPrevious', () => {
  it.each([
    [null, 0, false],
    [null, 2, true],
    [0, 3, false],
    [2, 0, true]
  ])('entry index %s with depth %s is %s', (index, depth, expected) => {
    expect(hasInAppPrevious(index, depth)).toBe(expected);
  });
});

describe('routeKey', () => {
  it('keeps the path and query and drops the hash', () => {
    expect(routeKey(new URL('http://localhost/folders?path=%2Fa#top'))).toBe('/folders?path=%2Fa');
  });
});

describe('navHistory without the Navigation API', () => {
  it('counts only navigations that grew the history', () => {
    navHistory.record(step('enter', null, '/library'));
    expect(navHistory.canPop()).toBe(false);

    push('link', '/library', '/people');
    expect(navHistory.canPop()).toBe(true);

    navHistory.record(step('popstate', '/people', '/library', -1));
    expect(navHistory.canPop()).toBe(false);
  });

  it('does not count a replacestate link', () => {
    navHistory.record(step('enter', null, '/settings'));
    navHistory.record(step('link', '/settings', '/settings/appearance'));
    expect(navHistory.canPop()).toBe(false);
  });

  it('does not count a cross-path goto that replaced the entry', () => {
    navHistory.record(step('enter', null, '/'));
    navHistory.record(step('goto', '/', '/photos'));
    expect(navHistory.canPop()).toBe(false);

    navHistory.back('/library');
    expect(browser.back).not.toHaveBeenCalled();
    expect(nav.goto).toHaveBeenCalledWith('/library', { replace: true });
  });

  it('counts the next push after a cancelled goto', () => {
    navHistory.record(step('enter', null, '/people/p1'));
    navHistory.back('/people');
    expect(nav.goto).toHaveBeenCalledWith('/people', { replace: true });

    push('link', '/people/p1', '/settings');
    expect(navHistory.canPop()).toBe(true);
  });

  it('tolerates navigation targets without a URL', () => {
    const entered = {
      ...step('goto', '/photos', '/people'),
      from: { url: null, params: null, route: { id: null } }
    } as unknown as AfterNavigate;
    browser.length += 1;

    expect(() => navHistory.record(entered)).not.toThrow();
    expect(navHistory.canPop()).toBe(true);
    expect(navHistory.previousIs('/photos')).toBe(false);
  });

  it('pops history when the previous entry is in the app', () => {
    navHistory.record(step('enter', null, '/library'));
    push('link', '/library', '/people');

    navHistory.back('/library');

    expect(browser.back).toHaveBeenCalledOnce();
    expect(nav.goto).not.toHaveBeenCalled();
  });

  it('goes up to the parent when the previous entry is outside the current stack', () => {
    navHistory.record(step('enter', null, '/photos'));
    push('link', '/photos', '/albums/trip');

    navHistory.back('/albums', (previous) => previous !== '/photos');

    expect(browser.back).not.toHaveBeenCalled();
    expect(nav.goto).toHaveBeenCalledWith('/albums', { replace: true });
  });

  it('remembers the pushed-from entry until history moves', () => {
    navHistory.record(step('enter', null, '/albums/x'));
    push('link', '/albums/x', '/assets/a?from=%2Falbums%2Fx');
    navHistory.record(step('goto', '/assets/a', '/assets/b?from=%2Falbums%2Fx'));
    expect(navHistory.previousIs('/albums/x')).toBe(true);
    expect(navHistory.previousIs('/albums/y')).toBe(false);

    navHistory.record(step('popstate', '/assets/b', '/albums/x', -1));
    expect(navHistory.previousIs('/albums/x')).toBe(false);
  });

  it('ignores the shallow entry a back layer pushes', () => {
    navHistory.record(step('enter', null, '/photos'));
    browser.length += 1;
    navHistory.record({ ...step('goto', '/photos', '/photos'), shallow: true });
    expect(navHistory.canPop()).toBe(false);

    push('link', '/photos', '/people');
    expect(navHistory.canPop()).toBe(true);
    expect(navHistory.previousIs('/photos')).toBe(true);
  });

  it('does not count a replace after a back layer entry as a push', () => {
    navHistory.record(step('enter', null, '/photos'));
    browser.length += 1;
    navHistory.record({ ...step('goto', '/photos', '/photos'), shallow: true });

    navHistory.record(step('goto', '/photos', '/people'));
    expect(navHistory.canPop()).toBe(false);
  });
});

describe('navHistory with the Navigation API', () => {
  it('trusts the entry index', () => {
    navHistory.record(step('enter', null, '/library'));
    push('link', '/library', '/people');
    stubNavigation(0, ['/people']);
    expect(navHistory.canPop()).toBe(false);

    stubNavigation(4, ['/', '/', '/', '/', '/people']);
    navHistory.record(step('enter', null, '/people'));
    expect(navHistory.canPop()).toBe(true);
  });

  it('reads the previous entry URL', () => {
    stubNavigation(1, ['/albums/x?sort=asc', '/assets/a']);
    expect(navHistory.previousIs('/albums/x?sort=asc')).toBe(true);
    expect(navHistory.previousIs('/albums/x')).toBe(false);

    stubNavigation(0, ['/assets/a']);
    expect(navHistory.previousIs('/albums/x')).toBe(false);
  });
});

describe('navHistory.returnTo', () => {
  it('pops back when the previous entry is the target and waits for arrival', async () => {
    stubNavigation(1, ['/albums/x', '/assets/a']);
    let arrived = false;
    const done = navHistory.returnTo('/albums/x').then(() => {
      arrived = true;
    });

    expect(browser.back).toHaveBeenCalledOnce();
    expect(nav.goto).not.toHaveBeenCalled();
    await Promise.resolve();
    expect(arrived).toBe(false);

    navHistory.record(step('popstate', '/assets/a', '/albums/x', -1));
    await done;
    expect(arrived).toBe(true);
  });

  it('replaces the entry when the previous entry is elsewhere', async () => {
    stubNavigation(1, ['/library', '/assets/a']);
    await navHistory.returnTo('/albums/x');

    expect(browser.back).not.toHaveBeenCalled();
    expect(nav.goto).toHaveBeenCalledWith('/albums/x', { replace: true });
  });
});
