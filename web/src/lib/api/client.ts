import { toasts } from '$lib/stores/toasts.svelte';

export class ApiError extends Error {
  status: number;
  code: string;
  requestId?: string;
  body?: unknown;
  constructor(status: number, code: string, message: string, requestId?: string, body?: unknown) {
    super(message);
    this.status = status;
    this.code = code;
    this.requestId = requestId;
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

const GATEWAY_STATUSES = new Set([0, 500, 502, 503, 504]);

export function isBackendDown(err: unknown): boolean {
  if (err instanceof NetworkError) return true;
  return err instanceof ApiError && GATEWAY_STATUSES.has(err.status);
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
  let requestId: string | undefined;
  let body: unknown;
  try {
    body = await resp.json();
    const b = body as { code?: unknown; message?: unknown; request_id?: unknown } | null;
    if (b && typeof b.code === 'string') code = b.code;
    if (b && typeof b.message === 'string') message = b.message;
    if (b && typeof b.request_id === 'string') requestId = b.request_id;
  } catch {
    /* ignore */
  }
  return new ApiError(resp.status, code, message, requestId, body);
}

function reportError(err: unknown): void {
  if (err instanceof ApiError) {
    if (err.status === 401 && err.code === 'unauthorized') {
      redirectToLogin();
      return;
    }
    if (err.code === 'upstream_unavailable') {
      toasts.push(
        'error',
        'Immich server unavailable. Check the server URL and that Immich is running.'
      );
    } else if (err.code === 'upstream_auth') {
      toasts.push('error', 'Immich rejected your session. Sign out and sign back in.');
    } else if (err.code === 'upstream_timeout') {
      toasts.push('warn', 'Immich request timed out.');
    } else if (err.status >= 500) {
      const reference = err.requestId ? ` Reference ${err.requestId}.` : '';
      toasts.push('error', `Server error. Check the immich-edit logs for details.${reference}`);
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

export function url(strings: TemplateStringsArray, ...values: unknown[]): string {
  return strings.reduce(
    (acc, part, i) => acc + (i > 0 ? encodeURIComponent(String(values[i - 1])) : '') + part,
    ''
  );
}

export async function request(
  path: string,
  init?: RequestInit,
  opts?: { silent?: boolean }
): Promise<Response> {
  const resp = await safeFetch(path, init);
  if (!resp.ok) {
    const err = await parseError(resp);
    if (!opts?.silent) reportError(err);
    throw err;
  }
  return resp;
}

export async function getJson<T>(
  path: string,
  init?: RequestInit,
  opts?: { silent?: boolean }
): Promise<T> {
  const resp = await request(path, init, opts);
  return (await resp.json()) as T;
}

export async function sendJson<T>(
  method: 'POST' | 'PUT' | 'PATCH' | 'DELETE',
  path: string,
  body: unknown,
  init?: RequestInit,
  opts?: { silent?: boolean }
): Promise<T> {
  const resp = await request(
    path,
    {
      ...init,
      method,
      headers: {
        'content-type': 'application/json',
        ...(init?.headers ?? {})
      },
      body: body === undefined ? undefined : JSON.stringify(body)
    },
    opts
  );
  if (resp.status === 204) return undefined as T;
  return (await resp.json()) as T;
}

export async function sendBytes<T>(path: string, bytes: Uint8Array): Promise<T> {
  const body = bytes.buffer.slice(
    bytes.byteOffset,
    bytes.byteOffset + bytes.byteLength
  ) as ArrayBuffer;
  const resp = await request(path, {
    method: 'POST',
    headers: { 'content-type': 'application/octet-stream' },
    body
  });
  return (await resp.json()) as T;
}

export async function postForBlob(
  path: string,
  body: unknown,
  signal?: AbortSignal
): Promise<{ blob: Blob; metaId: string | null }> {
  const resp = await request(path, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body),
    signal
  });
  const metaId = resp.headers.get('x-preview-meta-id');
  const blob = await resp.blob();
  return { blob, metaId };
}
