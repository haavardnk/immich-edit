import { describe, expect, it } from 'vitest';
import { ApiError, ConflictError, NetworkError, isBackendDown } from './client';

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
