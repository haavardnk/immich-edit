import { beforeEach, describe, expect, it, vi } from 'vitest';
import { rgbaToThumbHash } from 'thumbhash';
import { THUMBHASH_CACHE_SIZE, thumbhashDataUrl } from '#lib/utils/thumbhash.ts';

const decode = vi.hoisted(() => vi.fn<(hash: ArrayLike<number>) => string>());

vi.mock('thumbhash', async (importOriginal) => {
  const actual = await importOriginal<typeof import('thumbhash')>();
  decode.mockImplementation(actual.thumbHashToDataURL);
  return { ...actual, thumbHashToDataURL: decode };
});

function encode(red: number): string {
  const rgba = new Uint8Array(4 * 4 * 4);
  for (let i = 0; i < rgba.length; i += 4) rgba.set([red, 40, 90, 255], i);
  return btoa(String.fromCharCode(...rgbaToThumbHash(4, 4, rgba)));
}

describe('thumbhashDataUrl', () => {
  beforeEach(() => {
    decode.mockClear();
  });

  it('decodes a base64 thumbhash into a PNG data URL', () => {
    expect(thumbhashDataUrl(encode(200))).toMatch(/^data:image\/png;base64,/);
  });

  it('decodes each hash once', () => {
    const hash = encode(10);
    const first = thumbhashDataUrl(hash);
    expect(thumbhashDataUrl(hash)).toBe(first);
    expect(decode).toHaveBeenCalledTimes(1);
  });

  it('returns null for a hash that is not base64', () => {
    expect(thumbhashDataUrl('not base64!')).toBeNull();
    expect(thumbhashDataUrl('not base64!')).toBeNull();
    expect(decode).not.toHaveBeenCalled();
  });

  it('evicts the least recently used hash past the bound', () => {
    const kept = encode(20);
    const evicted = encode(30);
    thumbhashDataUrl(evicted);
    thumbhashDataUrl(kept);
    for (let i = 0; i < THUMBHASH_CACHE_SIZE - 1; i++) {
      thumbhashDataUrl(btoa(`filler-${i}`));
      thumbhashDataUrl(kept);
    }
    decode.mockClear();
    thumbhashDataUrl(kept);
    expect(decode).not.toHaveBeenCalled();
    thumbhashDataUrl(evicted);
    expect(decode).toHaveBeenCalledTimes(1);
  });
});
