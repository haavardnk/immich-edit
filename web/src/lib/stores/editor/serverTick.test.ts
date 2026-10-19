import { afterEach, describe, expect, it, vi } from 'vitest';
import { measureServerTick } from './serverTick';

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe('measureServerTick', () => {
  it('adds decode time to the server, encode and transfer time', async () => {
    const close = vi.fn();
    let now = 100;
    vi.spyOn(performance, 'now').mockImplementation(() => now);
    vi.stubGlobal(
      'createImageBitmap',
      vi.fn(async () => {
        now += 6;
        return { width: 1600, height: 1000, close };
      })
    );

    const sample = await measureServerTick(new Blob(), {
      renderMs: 20,
      encodeMs: 4,
      transferMs: 2
    });

    expect(sample).toEqual({ ms: 32, pixels: 1_600_000 });
    expect(close).toHaveBeenCalledOnce();
  });

  it('gives no sample when the preview cannot be decoded', async () => {
    vi.stubGlobal(
      'createImageBitmap',
      vi.fn(async () => {
        throw new Error('bad jpeg');
      })
    );

    expect(
      await measureServerTick(new Blob(), { renderMs: 1, encodeMs: 1, transferMs: 1 })
    ).toBeNull();
  });
});
