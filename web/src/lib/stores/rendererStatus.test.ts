import { describe, expect, it } from 'vitest';
import { rendererStatus, type StatusInput } from './rendererStatus';

const AUTO: StatusInput = { choice: 'auto', state: 'idle', reason: null, adapter: null };

describe('rendererStatus', () => {
  it('names the renderer and the reason', () => {
    expect(
      rendererStatus({
        ...AUTO,
        state: 'browser',
        adapter: 'Apple M2',
        reason: 'browser faster, 8.0 vs 30.0 ms/MP'
      })
    ).toBe('Browser on Apple M2: browser faster, 8.0 vs 30.0 ms/MP.');
    expect(rendererStatus({ ...AUTO, state: 'server', reason: 'no WebGPU' })).toBe(
      'Server: no WebGPU.'
    );
    expect(rendererStatus({ ...AUTO, state: 'measuring', reason: 'measuring speed' })).toBe(
      'Server: measuring speed.'
    );
  });

  it('keeps forced choices and waiting states short', () => {
    expect(rendererStatus({ ...AUTO, choice: 'server', state: 'server' })).toBe('Server.');
    expect(
      rendererStatus({ ...AUTO, choice: 'browser', state: 'browser', reason: 'browser selected' })
    ).toBe('Browser on the GPU.');
    expect(rendererStatus({ ...AUTO, state: 'starting' })).toBe('Starting the browser renderer.');
    expect(rendererStatus({ ...AUTO, choice: 'browser' })).toBe(
      'The browser renderer starts when a photo opens.'
    );
    expect(rendererStatus(AUTO)).toBe('Speed is measured when a photo opens.');
  });
});
