import { describe, it, expect, beforeEach, vi } from 'vitest';
import type { AssetDetail, TagRef } from '$lib/types/asset';

const tag: TagRef = { id: 't1', name: 'sunset', value: 'sunset' };
const h = vi.hoisted(() => ({
  updateAsset: vi.fn(),
  addTagToAsset: vi.fn(() => Promise.resolve())
}));

vi.mock('$lib/api/assets', () => ({ updateAsset: h.updateAsset, getAsset: vi.fn() }));
vi.mock('$lib/api/tags', () => ({
  addTagToAsset: h.addTagToAsset,
  removeTagFromAsset: vi.fn(() => Promise.resolve()),
  upsertTags: vi.fn(() => Promise.resolve([]))
}));
vi.mock('$lib/stores/metadataConsent.svelte', () => ({
  metadataConsent: { gate: () => Promise.resolve(true) }
}));

import { editor } from './editor.svelte';

function baseAsset(): AssetDetail {
  return {
    id: 'a1',
    originalFileName: 'IMG.ARW',
    type: 'IMAGE',
    originalMimeType: 'image/x-sony-arw',
    fileCreatedAt: null,
    updatedAt: null,
    checksum: null,
    isFavorite: false,
    exifInfo: null,
    tags: []
  };
}

describe('editor metadata mutations preserve tags', () => {
  beforeEach(() => {
    h.updateAsset.mockReset();
    h.addTagToAsset.mockClear();
    editor.assetId = 'a1';
    editor.asset = baseAsset();
  });

  it('keeps added tag after toggling favorite', async () => {
    await editor.addTag(tag);
    h.updateAsset.mockResolvedValueOnce({ ...baseAsset(), isFavorite: true, tags: [] });

    await editor.toggleFavorite();

    expect(editor.asset?.isFavorite).toBe(true);
    expect(editor.asset?.tags).toEqual([tag]);
  });

  it('keeps added tag after setting rating', async () => {
    await editor.addTag(tag);
    h.updateAsset.mockResolvedValueOnce({
      ...baseAsset(),
      exifInfo: { ...baseAsset().exifInfo, rating: 4 } as never,
      tags: []
    });

    await editor.setRating(4);

    expect(editor.asset?.exifInfo?.rating).toBe(4);
    expect(editor.asset?.tags).toEqual([tag]);
  });
});
