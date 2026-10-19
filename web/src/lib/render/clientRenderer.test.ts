import { afterEach, describe, expect, it, vi } from 'vitest';
import { ClientRenderer } from './clientRenderer';

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('ClientRenderer.start', () => {
  it.each([
    ['plain HTTP', false, { gpu: {} }, 'WebGPU needs HTTPS or localhost'],
    ['no WebGPU', true, {}, 'no WebGPU']
  ])('names the cause on %s', async (_case, secure, nav, message) => {
    vi.stubGlobal('isSecureContext', secure);
    vi.stubGlobal('navigator', nav);
    await expect(ClientRenderer.start(1600)).rejects.toThrow(message);
  });
});
