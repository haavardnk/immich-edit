import type { AssetSummary } from '#lib/types/album.ts';
import type { TagRef } from '#lib/types/asset.ts';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const h = vi.hoisted(() => ({
  addTagToAsset: vi.fn((_tagId: string, _assetId: string) => Promise.resolve()),
  removeTagFromAsset: vi.fn((_tagId: string, _assetId: string) => Promise.resolve()),
  getAsset: vi.fn((_id: string): Promise<{ tags: TagRef[] }> => Promise.resolve({ tags: [] }))
}));

vi.mock('#lib/api/tags.ts', () => ({
  addTagToAsset: h.addTagToAsset,
  removeTagFromAsset: h.removeTagFromAsset,
  upsertTags: vi.fn()
}));
vi.mock('#lib/api/assets.ts', () => ({ updateAsset: vi.fn(), getAsset: h.getAsset }));
vi.mock('#lib/stores/metadataConsent.svelte.ts', () => ({
  metadataConsent: { gate: () => Promise.resolve(true) }
}));
vi.mock('#lib/browse/reject.ts', async (orig) => {
  const actual = await orig<typeof import('#lib/browse/reject.ts')>();
  return {
    ...actual,
    ensureRejectTag: () => Promise.resolve({ id: 'r', name: 'reject', value: 'immich-edit/reject' })
  };
});

import { browseTarget, clearFlags, setLabel, toggleReject } from '#lib/browse/assetMeta.ts';
import { addTag, removeTag } from '#lib/browse/tagOps.ts';
import { browsing } from './browsing.svelte';
import { loupeTags } from './loupeTags.svelte';
import { rejected } from './rejected.svelte';
import { session } from './session.svelte';
import { tagCache } from './tagCache.svelte';

const sunset: TagRef = { id: 't1', name: 'sunset', value: 'sunset' };
const beach: TagRef = { id: 't2', name: 'beach', value: 'beach' };
const dusk: TagRef = { id: 't3', name: 'dusk', value: 'dusk' };

function summary(id: string): AssetSummary {
  return {
    id,
    originalFileName: id,
    type: 'IMAGE',
    fileCreatedAt: null,
    updatedAt: null,
    checksum: null,
    isFavorite: false,
    exifInfo: null,
    tags: []
  };
}

function ids(tags: TagRef[]): string[] {
  return tags.map((t) => t.id);
}

beforeEach(() => {
  h.addTagToAsset.mockClear();
  h.removeTagFromAsset.mockClear();
  session.clear();
  rejected.reset();
  browsing.set([summary('a')]);
  tagCache.set('a', [sunset, beach]);
});

describe('loupeTags', () => {
  it('keeps the photo tags through a reject and an unflag', async () => {
    expect(await toggleReject(loupeTags.target('a'))).toBe('saved');
    expect(ids(loupeTags.tagsOf('a'))).toEqual(['t1', 't2', 'r']);
    expect(rejected.ids.has('a')).toBe(true);
    expect(await clearFlags(loupeTags.target('a'))).toBe('saved');
    expect(ids(loupeTags.tagsOf('a'))).toEqual(['t1', 't2']);
  });

  it('un-rejects a photo rejected in the grid after its tags were cached', async () => {
    browsing.set([
      { ...summary('a'), tags: [{ id: 'r', name: 'reject', value: 'immich-edit/reject' }] }
    ]);
    expect(await toggleReject(loupeTags.target('a'))).toBe('saved');
    expect(h.removeTagFromAsset).toHaveBeenCalledWith('r', 'a');
    expect(ids(loupeTags.tagsOf('a'))).toEqual(['t1', 't2']);
  });

  it('replaces a label set in the grid after its tags were cached', async () => {
    const red = { id: 'red', name: 'red', value: 'immich-edit/label/red' };
    browsing.set([{ ...summary('a'), tags: [red] }]);
    expect(await setLabel(loupeTags.target('a'), null)).toBe('saved');
    expect(h.removeTagFromAsset).toHaveBeenCalledWith('red', 'a');
    expect(ids(browsing.find('a')?.tags ?? [])).toEqual(['t1', 't2']);
  });

  it('adds a tag to the cached tags and the list', async () => {
    tagCache.set('a', [sunset]);
    await loupeTags.add('a', beach);
    expect(h.addTagToAsset).toHaveBeenCalledWith('t2', 'a');
    expect(ids(loupeTags.tagsOf('a'))).toEqual(['t1', 't2']);
    expect(ids(browsing.find('a')?.tags ?? [])).toEqual(['t1', 't2']);
  });

  it('removes a tag the list summary never carried', async () => {
    await loupeTags.remove('a', 't1');
    expect(h.removeTagFromAsset).toHaveBeenCalledWith('t1', 'a');
    expect(ids(loupeTags.tagsOf('a'))).toEqual(['t2']);
  });

  it('keeps a tag removed outside the loupe out of the next loupe write', async () => {
    expect(await removeTag(browseTarget('a'), 't1')).toBe('saved');
    await loupeTags.add('a', dusk);
    expect(ids(loupeTags.tagsOf('a'))).toEqual(['t2', 't3']);
    expect(ids(browsing.find('a')?.tags ?? [])).toEqual(['t2', 't3']);
  });

  it('keeps a tag added outside the loupe in the next loupe write', async () => {
    expect(await addTag(browseTarget('a'), dusk)).toBe('saved');
    await loupeTags.remove('a', 't1');
    expect(ids(loupeTags.tagsOf('a'))).toEqual(['t2', 't3']);
    expect(ids(browsing.find('a')?.tags ?? [])).toEqual(['t2', 't3']);
  });

  it('keeps a tag added while the photo tags are still loading', async () => {
    browsing.set([summary('c')]);
    let finish: (tags: TagRef[]) => void = () => {};
    h.getAsset.mockImplementationOnce(
      () => new Promise((resolve) => (finish = (tags) => resolve({ tags })))
    );
    const loading = loupeTags.ensure('c');
    expect(await addTag(browseTarget('c'), dusk)).toBe('saved');
    finish([sunset]);
    await loading;
    expect(ids(loupeTags.tagsOf('c'))).toEqual(['t1', 't3']);
  });

  it('leaves an uncached photo to the list', async () => {
    browsing.set([summary('b')]);
    await loupeTags.add('b', sunset);
    expect(loupeTags.tagsOf('b')).toEqual([]);
    expect(ids(browsing.find('b')?.tags ?? [])).toEqual(['t1']);
  });
});
