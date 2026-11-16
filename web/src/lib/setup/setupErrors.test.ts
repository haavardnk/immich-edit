import { describe, expect, it } from 'vitest';
import { ApiError, NetworkError } from '#lib/api/client.ts';
import { claimError, probeError } from './setupErrors.ts';

function apiError(code: string, message = ''): ApiError {
  return new ApiError(400, code, message);
}

describe('claimError', () => {
  it('names known claim failures', () => {
    expect(claimError(apiError('admin_required'), 'x')).toMatch(/not an Immich administrator/);
    expect(claimError(apiError('conflict'), 'x')).toBe('This instance is already configured.');
  });

  it('falls back to the server message, then the fallback', () => {
    expect(claimError(apiError('other', 'boom'), 'x')).toBe('boom');
    expect(claimError(apiError('other'), 'Setup failed')).toBe('Setup failed');
  });

  it('reports a down immich-edit server', () => {
    expect(claimError(new NetworkError('down'), 'x')).toMatch(/not responding/);
  });
});

describe('probeError', () => {
  it('keeps the server reason for a rejected address', () => {
    expect(probeError(apiError('bad_request', 'immich_url must be http or https'))).toBe(
      'immich_url must be http or https'
    );
  });

  it('asks for a valid address otherwise', () => {
    expect(probeError(new Error('?'))).toBe('Enter a valid Immich URL.');
  });
});
