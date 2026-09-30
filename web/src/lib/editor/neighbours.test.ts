import { afterEach, describe, expect, it, vi } from 'vitest';
import { browsing } from '$lib/stores/browsing.svelte';
import type { AssetSummary } from '$lib/types/album';
import type { AssetType } from '$lib/types/asset';
import { warmNeighbours } from './neighbours';

function asset(id: string, type: AssetType = 'IMAGE'): AssetSummary {
  return {
    id,
    originalFileName: `${id}.arw`,
    type,
    fileCreatedAt: null,
    updatedAt: null,
    checksum: null,
    isFavorite: false,
    exifInfo: null,
    tags: []
  };
}

function stubFetch(): string[][] {
  const sent: string[][] = [];
  vi.stubGlobal(
    'fetch',
    vi.fn(async (_input: RequestInfo, init?: RequestInit) => {
      sent.push((JSON.parse(String(init?.body)) as { ids: string[] }).ids);
      return new Response(null, { status: 202 });
    })
  );
  return sent;
}

afterEach(() => {
  browsing.clear();
  vi.unstubAllGlobals();
});

describe('warmNeighbours', () => {
  it.each([
    ['both sides, next first', [asset('a'), asset('b'), asset('c')], 'b', [['c', 'a']]],
    ['images only', [asset('a', 'VIDEO'), asset('b'), asset('c')], 'b', [['c']]],
    ['nothing at the ends of a single asset', [asset('a')], 'a', []]
  ])('warms %s', (_case, assets, current, expected) => {
    const sent = stubFetch();
    browsing.set(assets);
    expect(warmNeighbours(current)).toBe(expected.length > 0);
    expect(sent).toEqual(expected);
  });
});
