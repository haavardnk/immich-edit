export type Strategy = 'network' | 'navigation' | 'precached' | 'runtime';

export type CacheRequest = {
  method: string;
  mode: string;
  url: string;
};

export const FALLBACK_PAGE = '/200.html';

const CACHE_PREFIX = 'immich-edit-';

const GATEWAY_FAILURES = new Set([502, 503, 504]);

export const SHELL_ASSETS = [
  FALLBACK_PAGE,
  '/manifest.webmanifest',
  '/favicon.svg',
  '/icon-192.png',
  '/icon-512.png',
  '/icon-maskable-512.png',
  '/apple-touch-icon.png'
];

const RUNTIME_PREFIXES = ['/fonts/', '/licenses/'];

function rooted(path: string): string {
  return path.startsWith('/') ? path : `/${path}`;
}

function isWasm(path: string): boolean {
  return path.endsWith('.wasm');
}

function isApi(path: string): boolean {
  return path === '/api' || path.startsWith('/api/');
}

function isRuntime(path: string): boolean {
  if (RUNTIME_PREFIXES.some((prefix) => path.startsWith(prefix))) return true;
  return path.startsWith('/_app/immutable/') && isWasm(path);
}

export function cacheName(version: string): string {
  return `${CACHE_PREFIX}${version}`;
}

export function isStaleCache(key: string, current: string): boolean {
  return key.startsWith(CACHE_PREFIX) && key !== current;
}

export function isGatewayFailure(status: number): boolean {
  return GATEWAY_FAILURES.has(status);
}

export function precachePaths(immutable: readonly { path: string }[]): string[] {
  const built = immutable.map(({ path }) => rooted(path)).filter((path) => !isWasm(path));
  return [...new Set([...built, ...SHELL_ASSETS])];
}

export function strategyFor(
  request: CacheRequest,
  origin: string,
  precached: ReadonlySet<string>
): Strategy {
  if (request.method !== 'GET') return 'network';
  const url = new URL(request.url);
  if (url.origin !== origin) return 'network';
  if (isApi(url.pathname)) return 'network';
  if (request.mode === 'navigate') return 'navigation';
  if (precached.has(url.pathname)) return 'precached';
  if (isRuntime(url.pathname)) return 'runtime';
  return 'network';
}
