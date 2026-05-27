import { toasts } from '$lib/stores/toasts.svelte';

export class ApiError extends Error {
  status: number;
  code: string;
  body?: unknown;
  constructor(status: number, code: string, message: string, body?: unknown) {
    super(message);
    this.status = status;
    this.code = code;
    this.body = body;
  }
}

export class ConflictError<T = unknown> extends Error {
  current?: T;
  constructor(message: string, current?: T) {
    super(message);
    this.current = current;
  }
}

export class NetworkError extends Error {
  constructor(message: string) {
    super(message);
  }
}

function redirectToLogin(): void {
  if (typeof window === 'undefined') return;
  if (window.location.pathname === '/login') return;
  const next = encodeURIComponent(window.location.pathname + window.location.search);
  window.location.replace(`/login?next=${next}`);
}

async function parseError(resp: Response): Promise<ApiError> {
  let code = 'unknown';
  let message = resp.statusText || 'request failed';
  let body: unknown;
  try {
    body = await resp.json();
    const b = body as { code?: unknown; message?: unknown } | null;
    if (b && typeof b.code === 'string') code = b.code;
    if (b && typeof b.message === 'string') message = b.message;
  } catch {
    /* ignore */
  }
  return new ApiError(resp.status, code, message, body);
}

function reportError(err: unknown): void {
  if (err instanceof ApiError) {
    if (err.status === 401 && err.code === 'unauthorized') {
      redirectToLogin();
      return;
    }
    if (err.code === 'upstream_unavailable') {
      toasts.push('error', 'Immich server unavailable. Check IMMICH_URL and that Immich is running.');
    } else if (err.code === 'upstream_auth') {
      toasts.push('error', 'Immich rejected the API key. Check IMMICH_API_KEY.');
    } else if (err.code === 'upstream_timeout') {
      toasts.push('warn', 'Immich request timed out.');
    } else if (err.status >= 500) {
      toasts.push('error', `Server error: ${err.message}`);
    }
    return;
  }
  if (err instanceof NetworkError) {
    toasts.push('error', 'Backend unreachable. Is immich-edit running?');
  }
}

async function safeFetch(input: RequestInfo, init?: RequestInit): Promise<Response> {
  try {
    return await fetch(input, { credentials: 'same-origin', ...init });
  } catch (err) {
    if (err instanceof DOMException && err.name === 'AbortError') throw err;
    const netErr = new NetworkError((err as Error)?.message ?? 'network error');
    reportError(netErr);
    throw netErr;
  }
}

export async function getJson<T>(path: string, init?: RequestInit): Promise<T> {
  const resp = await safeFetch(path, init);
  if (!resp.ok) {
    const err = await parseError(resp);
    reportError(err);
    throw err;
  }
  return (await resp.json()) as T;
}

export async function sendJson<T>(
  method: 'POST' | 'PUT' | 'DELETE',
  path: string,
  body: unknown,
  init?: RequestInit
): Promise<T> {
  const resp = await safeFetch(path, {
    ...init,
    method,
    headers: {
      'content-type': 'application/json',
      ...(init?.headers ?? {})
    },
    body: body === undefined ? undefined : JSON.stringify(body)
  });
  if (!resp.ok) {
    const err = await parseError(resp);
    reportError(err);
    throw err;
  }
  if (resp.status === 204) return undefined as T;
  return (await resp.json()) as T;
}

export async function postForBlob(
  path: string,
  body: unknown,
  signal?: AbortSignal
): Promise<{ blob: Blob; metaId: string | null }> {
  const resp = await safeFetch(path, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body),
    signal
  });
  if (!resp.ok) {
    const err = await parseError(resp);
    reportError(err);
    throw err;
  }
  const metaId = resp.headers.get('x-preview-meta-id');
  const blob = await resp.blob();
  return { blob, metaId };
}
