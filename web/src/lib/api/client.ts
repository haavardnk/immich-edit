export class ApiError extends Error {
  status: number;
  code: string;
  constructor(status: number, code: string, message: string) {
    super(message);
    this.status = status;
    this.code = code;
  }
}

async function parseError(resp: Response): Promise<ApiError> {
  let code = 'unknown';
  let message = resp.statusText || 'request failed';
  try {
    const body = await resp.json();
    if (typeof body?.code === 'string') code = body.code;
    if (typeof body?.message === 'string') message = body.message;
  } catch {
    /* ignore */
  }
  return new ApiError(resp.status, code, message);
}

export async function getJson<T>(path: string, init?: RequestInit): Promise<T> {
  const resp = await fetch(path, init);
  if (!resp.ok) throw await parseError(resp);
  return (await resp.json()) as T;
}

export async function sendJson<T>(
  method: 'POST' | 'PUT' | 'DELETE',
  path: string,
  body: unknown,
  init?: RequestInit
): Promise<T> {
  const resp = await fetch(path, {
    ...init,
    method,
    headers: {
      'content-type': 'application/json',
      ...(init?.headers ?? {})
    },
    body: body === undefined ? undefined : JSON.stringify(body)
  });
  if (!resp.ok) throw await parseError(resp);
  if (resp.status === 204) return undefined as T;
  return (await resp.json()) as T;
}

export async function postForBlob(
  path: string,
  body: unknown,
  signal?: AbortSignal
): Promise<{ blob: Blob; metaId: string | null }> {
  const resp = await fetch(path, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body),
    signal
  });
  if (!resp.ok) throw await parseError(resp);
  const metaId = resp.headers.get('x-preview-meta-id');
  const blob = await resp.blob();
  return { blob, metaId };
}
