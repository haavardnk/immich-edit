import { searchMetadata } from '#lib/api/search.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { photosAround, stillsOnly } from './trackPhotos';

vi.mock('#lib/api/search.ts', () => ({ searchMetadata: vi.fn() }));

const search = vi.mocked(searchMetadata);

function asset(id: string, type: AssetSummary['type']): AssetSummary {
  return { id, type } as AssetSummary;
}

describe('track photos', () => {
  beforeEach(() => search.mockReset());

  it('asks the library for images only', async () => {
    search.mockResolvedValue({ items: [asset('a', 'IMAGE')], nextCursor: null } as never);
    const found = await photosAround(Date.UTC(2026, 9, 31, 9), Date.UTC(2026, 9, 31, 10));
    expect(found.map((item) => item.id)).toEqual(['a']);
    expect(search.mock.calls[0]![0].filter).toMatchObject({ imagesOnly: true });
  });

  it('leaves videos out of a selection', () => {
    const picked = [asset('a', 'IMAGE'), asset('v', 'VIDEO'), asset('b', 'IMAGE')];
    expect(stillsOnly(picked).map((item) => item.id)).toEqual(['a', 'b']);
  });
});
