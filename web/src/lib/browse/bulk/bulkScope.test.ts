import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { session } from '#lib/stores/session.svelte.ts';
import { selection } from '#lib/stores/selection.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import { bulkScope, type BulkSource } from './bulkScope.svelte';

function asset(id: string, fields: Partial<AssetSummary> = {}): AssetSummary {
  return {
    id,
    originalFileName: `${id}.jpg`,
    type: 'IMAGE',
    fileCreatedAt: null,
    updatedAt: null,
    checksum: null,
    isFavorite: false,
    exifInfo: null,
    tags: [],
    ...fields
  };
}

const ITEMS = [
  asset('a', { isFavorite: true, exifInfo: { rating: 3 } as AssetSummary['exifInfo'] }),
  asset('b', { isFavorite: true, exifInfo: { rating: 3 } as AssetSummary['exifInfo'] }),
  asset('c', { ownerId: 'partner' })
];

let detach = (): void => {};

function source(fields: Partial<BulkSource> = {}): BulkSource {
  return {
    items: () => ITEMS,
    selectingAll: () => false,
    selectAll: async () => true,
    partial: () => false,
    loadingMore: () => false,
    total: () => ITEMS.length,
    ...fields
  };
}

beforeEach(() => {
  session.set({ id: 'me', email: 'me@x', name: 'Me', is_admin: false, auth_kind: 'password' });
  selection.clear();
});

afterEach(() => detach());

describe('bulkScope', () => {
  it('is not selecting without a grid even when photos are picked', () => {
    selection.replace(['a']);
    expect(bulkScope.selecting).toBe(false);
    detach = bulkScope.attach(source());
    expect(bulkScope.selecting).toBe(true);
  });

  it('counts select mode with nothing picked as selecting', () => {
    detach = bulkScope.attach(source());
    selection.enterMode();
    expect(bulkScope.selecting).toBe(true);
    expect(bulkScope.count).toBe(0);
    expect(bulkScope.closeLabel).toBe('Exit selection');
    selection.toggle('a');
    expect(bulkScope.closeLabel).toBe('Clear selection');
  });

  it('explains why only shared photos cannot be trashed', () => {
    detach = bulkScope.attach(source());
    selection.replace(['c']);
    expect(bulkScope.trashable).toBe(false);
    expect(bulkScope.trashReason).toMatch(/Only the owner/);
    selection.toggle('a');
    expect(bulkScope.trashReason).toBeNull();
  });

  it('reads shared values over the picked photos', () => {
    detach = bulkScope.attach(source());
    selection.replace(['a', 'b']);
    expect(bulkScope.picked.map((item) => item.id)).toEqual(['a', 'b']);
    expect(bulkScope.favorite).toBe('all');
    expect(bulkScope.rating).toBe(3);
    expect(bulkScope.allOwned).toBe(true);
    expect(bulkScope.single).toBeNull();
    selection.toggle('c');
    expect(bulkScope.favorite).toBe('some');
    expect(bulkScope.rating).toBeNull();
    expect(bulkScope.allOwned).toBe(false);
    expect(bulkScope.trashable).toBe(true);
  });

  it('names the one picked photo', () => {
    detach = bulkScope.attach(source());
    selection.replace(['b']);
    expect(bulkScope.single?.id).toBe('b');
  });

  it.each([
    ['everything loaded and picked', false, ['a', 'b', 'c'], false, null],
    ['some loaded photos unpicked', false, ['a'], true, null],
    ['more pages to load', true, ['a', 'b', 'c'], true, '3 of 40 loaded']
  ])('with %s offers Select all and reads the scope', (_name, partial, picked, all, scope) => {
    detach = bulkScope.attach(source({ partial: () => partial, total: () => 40 }));
    selection.replace(picked);
    expect(bulkScope.canSelectAll).toBe(all);
    expect(bulkScope.scope).toBe(partial ? scope : null);
  });

  it('targets every listed photo while selecting all', () => {
    detach = bulkScope.attach(source({ selectingAll: () => true }));
    expect(bulkScope.targetCount).toBe(3);
    expect(bulkScope.selecting).toBe(true);
  });

  it('stops reading a grid once it detaches', () => {
    const stop = bulkScope.attach(source());
    stop();
    expect(bulkScope.items).toEqual([]);
  });
});
