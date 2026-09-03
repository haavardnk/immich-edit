import { afterEach, describe, expect, it, vi } from 'vitest';
import { ApiError, ConflictError, NetworkError, isBackendDown, request, url } from './client';
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

describe('url', () => {
  it.each([
    ['../../etc/passwd', '/api/assets/..%2F..%2Fetc%2Fpasswd/edits'],
    ['a?b=c#d', '/api/assets/a%3Fb%3Dc%23d/edits'],
    ['spaced id', '/api/assets/spaced%20id/edits'],
    ['smør&ost', '/api/assets/sm%C3%B8r%26ost/edits'],
    [
      '0f0d4ae1-1111-4222-8333-444455556666',
      '/api/assets/0f0d4ae1-1111-4222-8333-444455556666/edits'
    ]
  ])('encodes %s into a single path segment', (id, expected) => {
    expect(url`/api/assets/${id}/edits`).toBe(expected);
  });

  it('leaves literal separators alone and encodes query values', () => {
    expect(url`/api/assets/${'a b'}/thumb?size=${'x&y'}`).toBe(
      '/api/assets/a%20b/thumb?size=x%26y'
    );
  });
});
