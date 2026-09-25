import { afterEach, describe, expect, it, vi } from 'vitest';
import { ClientRenderer } from './client-renderer';

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('ClientRenderer.start', () => {
  it.each([
    ['plain HTTP', false, { gpu: {} }, 'browsers only offer WebGPU over HTTPS or on localhost'],
    ['no WebGPU', true, {}, 'this browser has no WebGPU']
  ])('names the cause on %s', async (_case, secure, nav, message) => {
    vi.stubGlobal('isSecureContext', secure);
    vi.stubGlobal('navigator', nav);
    await expect(ClientRenderer.start()).rejects.toThrow(message);
  });
});
