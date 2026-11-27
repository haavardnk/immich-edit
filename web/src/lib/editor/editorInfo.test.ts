import { editorInfoAsset } from '#lib/editor/editorInfo.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import type { AssetDetail } from '#lib/types/asset.ts';
import { afterEach, describe, expect, it } from 'vitest';

function summary(id: string): AssetSummary {
  return {
    id,
    originalFileName: `${id}.ARW`,
    type: 'IMAGE',
    fileCreatedAt: null,
    updatedAt: null,
    checksum: null,
    isFavorite: false,
    exifInfo: null,
    tags: []
  };
}

describe('editorInfoAsset', () => {
  afterEach(() => {
    editor.asset = null;
    editor.assetId = null;
    browsing.clear();
  });

  it('shows the listed photo while the editor loads its detail', () => {
    browsing.set([summary('a1'), summary('a2')]);
    editor.assetId = 'a2';

    expect(editorInfoAsset()?.originalFileName).toBe('a2.ARW');
  });

  it('prefers the loaded detail', () => {
    browsing.set([summary('a1')]);
    editor.assetId = 'a1';
    editor.asset = { ...summary('a1'), originalFileName: 'loaded.ARW' } as AssetDetail;

    expect(editorInfoAsset()?.originalFileName).toBe('loaded.ARW');
  });

  it('has nothing before the editor opens a photo', () => {
    browsing.set([summary('a1')]);

    expect(editorInfoAsset()).toBeNull();
  });
});
