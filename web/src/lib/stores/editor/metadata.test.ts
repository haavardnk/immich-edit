import { describe, it, expect, beforeEach, vi } from 'vitest';
import type { AssetDetail, TagRef } from '#lib/types/asset.ts';

const tag: TagRef = { id: 't1', name: 'sunset', value: 'sunset' };
const h = vi.hoisted(() => ({
  updateAsset: vi.fn(),
  addTagToAsset: vi.fn(() => Promise.resolve())
}));

vi.mock('#lib/api/assets.ts', () => ({ updateAsset: h.updateAsset, getAsset: vi.fn() }));
vi.mock('#lib/api/tags.ts', () => ({
  addTagToAsset: h.addTagToAsset,
  removeTagFromAsset: vi.fn(() => Promise.resolve()),
  upsertTags: vi.fn(() => Promise.resolve([]))
}));
vi.mock('#lib/stores/metadataConsent.svelte.ts', () => ({
  metadataConsent: { gate: () => Promise.resolve(true) }
}));

import { editor } from '#lib/stores/editor.svelte.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { session } from '#lib/stores/session.svelte.ts';

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
    editor.error = null;
    session.clear();
    browsing.set([{ ...baseAsset() }]);
  });

  it('writes the rating to the open photo and the browse list', async () => {
    await editor.setRating(3);

    expect(h.updateAsset).toHaveBeenCalledWith('a1', { rating: 3 });
    expect(editor.asset?.exifInfo?.rating).toBe(3);
    expect(browsing.assets[0]?.exifInfo?.rating).toBe(3);
  });

  it('restores the photo and shows why when Immich refuses', async () => {
    h.updateAsset.mockRejectedValueOnce(new Error('Immich said no'));

    await editor.toggleFavorite();

    expect(editor.asset?.isFavorite).toBe(false);
    expect(browsing.assets[0]?.isFavorite).toBe(false);
    expect(editor.error).toBe('favorite: Immich said no');
  });

  it('leaves the next photo alone when a write fails after moving on', async () => {
    let refuse: (error: Error) => void = () => {};
    h.updateAsset.mockReturnValueOnce(new Promise((_, reject) => (refuse = reject)));
    const pending = editor.toggleFavorite();
    await vi.waitFor(() => expect(h.updateAsset).toHaveBeenCalled());
    editor.assetId = 'a2';
    editor.asset = { ...baseAsset(), id: 'a2' };

    refuse(new Error('Immich said no'));
    await pending;

    expect(editor.asset?.isFavorite).toBe(false);
    expect(editor.error).toBeNull();
    expect(browsing.assets[0]?.isFavorite).toBe(false);
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

  it('leaves photos owned by another Immich user untouched', async () => {
    session.set({ id: 'me', email: 'me@x', name: 'Me', is_admin: false, auth_kind: 'password' });
    editor.asset = { ...baseAsset(), ownerId: 'partner' };

    await editor.toggleFavorite();
    await editor.addTag(tag);

    expect(h.updateAsset).not.toHaveBeenCalled();
    expect(h.addTagToAsset).not.toHaveBeenCalled();
    expect(editor.asset?.isFavorite).toBe(false);
    expect(editor.asset?.tags).toEqual([]);
  });
});
