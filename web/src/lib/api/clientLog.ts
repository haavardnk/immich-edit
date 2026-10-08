import { errorMessage } from '$lib/utils/errors';
import { ApiError, NetworkError } from './client';

export type ClientLogKind =
  'exception' | 'unhandled_rejection' | 'svelte' | 'render_worker' | 'gpu';
export type ClientLogLevel = 'error' | 'warn';

const MAX_PER_PAGE = 20;
const DEDUPE_MS = 60_000;
const MAX_MESSAGE = 2000;
const MAX_STACK = 8000;

const lastSent = new Map<string, number>();
let sentCount = 0;
let installed = false;

export function reportClientError(
  kind: ClientLogKind,
  err: unknown,
  level: ClientLogLevel = 'error'
): void {
  if (err instanceof ApiError || err instanceof NetworkError) return;
  if (err instanceof DOMException && err.name === 'AbortError') return;
  if (sentCount >= MAX_PER_PAGE) return;
  const message = errorMessage(err).slice(0, MAX_MESSAGE);
  const key = `${kind}|${message}`;
  const now = Date.now();
  const previous = lastSent.get(key);
  if (previous !== undefined && now - previous < DEDUPE_MS) return;
  lastSent.set(key, now);
  sentCount++;
  const body = {
    kind,
    level,
    message,
    stack: err instanceof Error ? err.stack?.slice(0, MAX_STACK) : undefined,
    route: location.pathname
  };
  void fetch('/api/client-log', {
    method: 'POST',
    credentials: 'same-origin',
    keepalive: true,
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body)
  }).catch(() => undefined);
}

export function installClientLog(): void {
  if (installed) return;
  installed = true;
  window.addEventListener('error', (event) =>
    reportClientError('exception', event.error ?? event.message)
  );
  window.addEventListener('unhandledrejection', (event) =>
    reportClientError('unhandled_rejection', event.reason)
  );
}
