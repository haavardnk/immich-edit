import { afterEach, describe, expect, it, vi } from 'vitest';
import { ApiError, ConflictError, NetworkError, isBackendDown, request } from './client';
import { toasts } from '$lib/stores/toasts.svelte';

describe('isBackendDown', () => {
  it.each([0, 500, 502, 503, 504])('treats status %i as down', (status) => {
    expect(isBackendDown(new ApiError(status, 'unknown', 'Bad Gateway'))).toBe(true);
  });

  it.each([400, 401, 404, 409, 429])('treats status %i as up', (status) => {
    expect(isBackendDown(new ApiError(status, 'unknown', 'nope'))).toBe(false);
  });

  it('treats a rejected fetch as down', () => {
    expect(isBackendDown(new NetworkError('failed to fetch'))).toBe(true);
  });

  it('ignores unrelated errors', () => {
    expect(isBackendDown(new ConflictError('conflict'))).toBe(false);
    expect(isBackendDown(new Error('boom'))).toBe(false);
    expect(isBackendDown(null)).toBe(false);
  });
});

describe('server error reporting', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    toasts.items = [];
  });

  it('never renders the server message and quotes the request id', async () => {
    vi.stubGlobal('fetch', async () =>
      Response.json(
        {
          code: 'internal',
          message: '/var/lib/immich-edit/data.db: no such column: foo',
          request_id: 'abc-123'
        },
        { status: 500 }
      )
    );

    await expect(request('/api/edits')).rejects.toBeInstanceOf(ApiError);

    const message = toasts.items.at(-1)?.message ?? '';
    expect(message).not.toContain('no such column');
    expect(message).toContain('abc-123');
  });
});
