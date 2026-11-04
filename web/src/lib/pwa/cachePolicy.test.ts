import { existsSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import {
  cacheName,
  FALLBACK_PAGE,
  isGatewayFailure,
  isStaleCache,
  precachePaths,
  SHELL_ASSETS,
  strategyFor,
  type CacheRequest
} from '#lib/pwa/cachePolicy.ts';

const ORIGIN = 'https://edit.example';
const PRECACHED = new Set(['/_app/immutable/entry/start.js', FALLBACK_PAGE]);

function get(path: string, mode = 'cors'): CacheRequest {
  return { method: 'GET', mode, url: `${ORIGIN}${path}` };
}

describe('precachePaths', () => {
  it('roots built paths, drops the wasm and adds the shell', () => {
    const paths = precachePaths([
      { path: '_app/immutable/entry/start.js' },
      { path: '/_app/immutable/chunks/a.js' },
      { path: '_app/immutable/workers/assets/web_render_bg-abc.wasm' }
    ]);

    expect(paths).toContain('/_app/immutable/entry/start.js');
    expect(paths).toContain('/_app/immutable/chunks/a.js');
    expect(paths.some((path) => path.endsWith('.wasm'))).toBe(false);
    expect(paths).toEqual(
      expect.arrayContaining([FALLBACK_PAGE, '/manifest.webmanifest', '/apple-touch-icon.png'])
    );
  });

  it('lists each path once', () => {
    const paths = precachePaths([{ path: '/200.html' }]);
    expect(paths.filter((path) => path === FALLBACK_PAGE)).toHaveLength(1);
  });
});

describe('SHELL_ASSETS', () => {
  it.each(SHELL_ASSETS.filter((path) => path !== FALLBACK_PAGE))('%s ships in static', (path) => {
    expect(existsSync(join(process.cwd(), 'static', path))).toBe(true);
  });
});

describe('cache names', () => {
  it('prefixes the version', () => {
    expect(cacheName('123')).toBe('immich-edit-123');
  });

  it('drops only older caches it owns', () => {
    const current = cacheName('2');
    expect(isStaleCache(cacheName('1'), current)).toBe(true);
    expect(isStaleCache(current, current)).toBe(false);
    expect(isStaleCache('cache-1', current)).toBe(false);
    expect(isStaleCache('workbox-precache', current)).toBe(false);
  });
});

describe('isGatewayFailure', () => {
  it.each([502, 503, 504])('falls back on %i', (status) => {
    expect(isGatewayFailure(status)).toBe(true);
  });

  it.each([200, 404, 500])('passes %i through', (status) => {
    expect(isGatewayFailure(status)).toBe(false);
  });
});

describe('strategyFor', () => {
  it.each(['/api', '/api/assets?x=1', '/api/auth/session'])('never handles %s', (path) => {
    expect(strategyFor(get(path), ORIGIN, PRECACHED)).toBe('network');
    expect(strategyFor(get(path, 'navigate'), ORIGIN, PRECACHED)).toBe('network');
  });

  it('leaves non-GET and cross-origin requests to the network', () => {
    const post = { ...get('/_app/immutable/entry/start.js'), method: 'POST' };
    const foreign = { ...get('/fonts/a.woff2'), url: 'https://cdn.example/fonts/a.woff2' };

    expect(strategyFor(post, ORIGIN, PRECACHED)).toBe('network');
    expect(strategyFor(foreign, ORIGIN, PRECACHED)).toBe('network');
  });

  it('serves navigations network first', () => {
    expect(strategyFor(get('/edit/abc', 'navigate'), ORIGIN, PRECACHED)).toBe('navigation');
  });

  it('serves precached files from the cache', () => {
    expect(strategyFor(get('/_app/immutable/entry/start.js'), ORIGIN, PRECACHED)).toBe('precached');
  });

  it.each([
    '/fonts/inter.woff2',
    '/licenses/third-party.txt',
    '/_app/immutable/workers/assets/web_render_bg-abc.wasm'
  ])('caches %s at runtime', (path) => {
    expect(strategyFor(get(path), ORIGIN, PRECACHED)).toBe('runtime');
  });

  it('leaves anything else to the network', () => {
    expect(strategyFor(get('/apiary.png'), ORIGIN, PRECACHED)).toBe('network');
    expect(strategyFor(get('/robots.txt'), ORIGIN, PRECACHED)).toBe('network');
  });
});
