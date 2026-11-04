import { asset } from '#lib/testing/assets.ts';
import type { TagRef } from '#lib/types/asset.ts';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const h = vi.hoisted(() => ({
  addTagToAsset: vi.fn((_tagId: string, _assetId: string) => Promise.resolve()),
  removeTagFromAsset: vi.fn((_tagId: string, _assetId: string) => Promise.resolve()),
  updateAsset: vi.fn((_id: string, _fields: object) => Promise.resolve()),
  gate: vi.fn(() => Promise.resolve(true)),
  rejectTag: { id: 'r', name: 'reject', value: 'immich-edit/reject' } as {
    id: string;
    name: string;
    value: string;
  } | null
}));

vi.mock('#lib/api/tags.ts', () => ({
  addTagToAsset: h.addTagToAsset,
  removeTagFromAsset: h.removeTagFromAsset,
  upsertTags: vi.fn()
}));
vi.mock('#lib/api/assets.ts', () => ({ updateAsset: h.updateAsset }));
vi.mock('#lib/stores/metadataConsent.svelte.ts', () => ({ metadataConsent: { gate: h.gate } }));
vi.mock('#lib/browse/reject.ts', async (orig) => {
  const actual = await orig<typeof import('#lib/browse/reject.ts')>();
  return { ...actual, ensureRejectTag: () => Promise.resolve(h.rejectTag) };
});
vi.mock('#lib/browse/managedTags.ts', async (orig) => {
  const actual = await orig<typeof import('#lib/browse/managedTags.ts')>();
  return {
    ...actual,
    ensureManagedTag: (value: string) =>
      Promise.resolve({ id: value.split('/').at(-1) ?? value, name: value, value })
  };
});

import {
  addTags,
  removeTags,
  setFavorite,
  setLabel,
  setRating,
  setRejected
} from '#lib/browse/bulkMeta.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { labelMembers } from '#lib/stores/labels.svelte.ts';
import { labelOf } from '#lib/stores/labels.ts';
import { rejected } from '#lib/stores/rejected.svelte.ts';
import { session } from '#lib/stores/session.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';

const sunset: TagRef = { id: 't1', name: 'sunset', value: 'sunset' };
const beach: TagRef = { id: 't2', name: 'beach', value: 'beach' };
const push = vi.spyOn(toasts, 'push');

function tagIds(id: string): string[] {
  return (browsing.find(id)?.tags ?? []).map((t) => t.id);
}

beforeEach(() => {
  h.addTagToAsset.mockReset();
  h.addTagToAsset.mockResolvedValue();
  h.removeTagFromAsset.mockReset();
  h.removeTagFromAsset.mockResolvedValue();
  h.updateAsset.mockReset();
  h.updateAsset.mockResolvedValue();
  h.gate.mockReset();
  h.gate.mockResolvedValue(true);
  push.mockReset();
  push.mockImplementation(() => {});
  h.rejectTag = { id: 'r', name: 'reject', value: 'immich-edit/reject' };
  session.clear();
  rejected.reset();
  for (const members of Object.values(labelMembers)) members.reset();
  browsing.set([asset('a'), asset('b'), asset('c')]);
});

describe('bulk metadata', () => {
  it('writes every photo', async () => {
    expect(await setRating(['a', 'b', 'c'], 4)).toBe(true);
    expect(h.updateAsset).toHaveBeenCalledTimes(3);
    expect(browsing.assets.map((a) => a.exifInfo?.rating)).toEqual([4, 4, 4]);
    expect(push).toHaveBeenCalledWith('success', 'Updated 3 assets', { ttlMs: 4000 });
  });

  it('writes nothing when consent is refused', async () => {
    h.gate.mockResolvedValue(false);
    expect(await setFavorite(['a', 'b'], true)).toBe(false);
    expect(h.updateAsset).not.toHaveBeenCalled();
    expect(push).not.toHaveBeenCalled();
  });

  it('counts the photos Immich refused', async () => {
    h.updateAsset.mockImplementation((id) =>
      id === 'b' ? Promise.reject(new Error('boom')) : Promise.resolve()
    );
    expect(await setFavorite(['a', 'b', 'c'], true)).toBe(true);
    expect(browsing.assets.map((a) => a.isFavorite)).toEqual([true, false, true]);
    expect(push).toHaveBeenCalledWith('warn', '1 of 3 failed', { ttlMs: 6000 });
  });

  it('counts photos it could not write as failed', async () => {
    browsing.set([asset('a')]);
    expect(await setRating(['a', 'gone'], 2)).toBe(true);
    expect(push).toHaveBeenCalledWith('warn', '1 of 2 failed', { ttlMs: 6000 });
  });

  it('rejects every photo', async () => {
    expect(await setRejected(['a', 'b'], true)).toBe(true);
    expect(h.addTagToAsset).toHaveBeenCalledTimes(2);
    expect(rejected.ids.has('a') && rejected.ids.has('b')).toBe(true);
    expect(push).toHaveBeenCalledWith('success', 'Rejected 2 assets', { ttlMs: 4000 });
  });

  it('stops when the reject tag cannot be created', async () => {
    h.rejectTag = null;
    expect(await setRejected(['a', 'b'], true)).toBe(false);
    expect(h.addTagToAsset).not.toHaveBeenCalled();
    expect(push).toHaveBeenCalledWith('error', 'reject: could not create tag', { ttlMs: 6000 });
  });

  it('labels every photo', async () => {
    expect(await setLabel(['a', 'b'], 'red')).toBe(true);
    expect(browsing.assets.slice(0, 2).map(labelOf)).toEqual(['red', 'red']);
    expect(push).toHaveBeenCalledWith('success', 'Red label on 2 photos', { ttlMs: 4000 });
  });
});

describe('bulk tags', () => {
  it('adds every tag to every photo', async () => {
    expect(await addTags(['a', 'b'], [sunset, beach])).toBe(true);
    expect(h.addTagToAsset).toHaveBeenCalledTimes(4);
    expect(tagIds('a')).toEqual(['t1', 't2']);
    expect(tagIds('b')).toEqual(['t1', 't2']);
    expect(push).toHaveBeenCalledWith('success', 'Added tags on 2 assets', { ttlMs: 4000 });
  });

  it('removes every tag from every photo', async () => {
    browsing.set([asset('a', { tags: [sunset, beach] }), asset('b', { tags: [sunset] })]);
    expect(await removeTags(['a', 'b'], [sunset])).toBe(true);
    expect(tagIds('a')).toEqual(['t2']);
    expect(tagIds('b')).toEqual([]);
    expect(push).toHaveBeenCalledWith('success', 'Removed tags on 2 assets', { ttlMs: 4000 });
  });

  it('counts each refused tag update', async () => {
    h.addTagToAsset.mockImplementation((tagId, assetId) =>
      tagId === 't2' && assetId === 'b' ? Promise.reject(new Error('boom')) : Promise.resolve()
    );
    expect(await addTags(['a', 'b'], [sunset, beach])).toBe(true);
    expect(tagIds('b')).toEqual(['t1']);
    expect(push).toHaveBeenCalledWith('warn', '1 tag update failed', { ttlMs: 6000 });
  });

  it('writes nothing when consent is refused', async () => {
    h.gate.mockResolvedValue(false);
    expect(await addTags(['a'], [sunset])).toBe(false);
    expect(h.addTagToAsset).not.toHaveBeenCalled();
  });
});
