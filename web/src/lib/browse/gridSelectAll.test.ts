import { beforeEach, describe, expect, it } from 'vitest';
import { selection } from '#lib/stores/selection.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import { GridSelectAll } from './gridSelectAll.svelte';

function asset(id: string): AssetSummary {
  return {
    id,
    originalFileName: `${id}.jpg`,
    type: 'IMAGE',
    fileCreatedAt: null,
    updatedAt: null,
    checksum: null,
    isFavorite: false,
    exifInfo: null,
    tags: []
  };
}

function grid(loads: boolean, loadingMore = false) {
  let items = ['a', 'b'].map(asset);
  const whole = new GridSelectAll({
    items: () => items,
    loadingMore: () => loadingMore,
    loadAll: () => async () => {
      expect(whole.busy).toBe(true);
      if (loads) items = ['a', 'b', 'c', 'd'].map(asset);
      return loads;
    }
  });
  return whole;
}

beforeEach(() => selection.clear());

describe('GridSelectAll', () => {
  it('loads the whole view before selecting all of it', async () => {
    const whole = grid(true);
    expect(await whole.run()).toBe(true);
    expect([...selection.selected]).toEqual(['a', 'b', 'c', 'd']);
    expect(whole.busy).toBe(false);
  });

  it('keeps the selection when the view fails to load', async () => {
    selection.replace(['b']);
    expect(await grid(false).run()).toBe(false);
    expect([...selection.selected]).toEqual(['b']);
  });

  it('waits for a page that is already loading', async () => {
    selection.replace(['b']);
    expect(await grid(true, true).run()).toBe(false);
    expect([...selection.selected]).toEqual(['b']);
  });
});
