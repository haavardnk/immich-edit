import { self } from '$app/service-worker';
import { version } from '$app/env';
import { immutable } from '$app/manifest';
import {
  cacheName,
  FALLBACK_PAGE,
  isGatewayFailure,
  isStaleCache,
  precachePaths,
  strategyFor
} from '#lib/pwa/cachePolicy.ts';
import { isSkipWaiting } from '#lib/pwa/protocol.ts';

const CACHE = cacheName(version);
const PRECACHE = precachePaths(immutable);
const PRECACHED = new Set(PRECACHE);

async function precache(): Promise<void> {
  const cache = await caches.open(CACHE);
  await cache.addAll(PRECACHE);
}

async function dropOldCaches(): Promise<void> {
  const keys = await caches.keys();
  const stale = keys.filter((key) => isStaleCache(key, CACHE));
  await Promise.all(stale.map((key) => caches.delete(key)));
}

async function fallbackPage(): Promise<Response | undefined> {
  const page = await caches.match(FALLBACK_PAGE, { cacheName: CACHE });
  if (!page?.redirected) return page;
  return new Response(page.body, {
    status: page.status,
    statusText: page.statusText,
    headers: page.headers
  });
}

async function navigate(request: Request): Promise<Response> {
  let response: Response;
  try {
    response = await fetch(request);
  } catch (err: unknown) {
    const fallback = await fallbackPage();
    if (fallback) return fallback;
    throw err;
  }
  if (!isGatewayFailure(response.status)) return response;
  return (await fallbackPage()) ?? response;
}

async function cacheFirst(event: FetchEvent, store: boolean): Promise<Response> {
  const { request } = event;
  const cache = await caches.open(CACHE);
  const hit = await cache.match(request);
  if (hit) return hit;
  const response = await fetch(request);
  if (store && response.status === 200 && response.type === 'basic') {
    event.waitUntil(cache.put(request, response.clone()).catch(() => undefined));
  }
  return response;
}

self.addEventListener('install', (event) => {
  event.waitUntil(precache());
});

self.addEventListener('activate', (event) => {
  event.waitUntil(dropOldCaches());
});

self.addEventListener('message', (event) => {
  if (isSkipWaiting(event.data)) void self.skipWaiting();
});

self.addEventListener('fetch', (event) => {
  const strategy = strategyFor(event.request, self.location.origin, PRECACHED);
  if (strategy === 'network') return;
  if (strategy === 'navigation') event.respondWith(navigate(event.request));
  else event.respondWith(cacheFirst(event, strategy === 'runtime'));
});
