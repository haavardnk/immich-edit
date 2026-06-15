import { describe, it, expect, beforeEach, vi } from 'vitest';
import type { AssetSummary } from '$lib/types/album';
import type { TagRef } from '$lib/types/asset';

const rejectTag: TagRef = { id: 'r', name: 'reject', value: 'immich-edit/reject' };
const h = vi.hoisted(() => ({
  addTagToAsset: vi.fn(() => Promise.resolve()),
  removeTagFromAsset: vi.fn(() => Promise.resolve()),
  updateAsset: vi.fn(() => Promise.resolve({} as never)),
  consent: { value: true },
  rejectTag: { id: 'r', name: 'reject', value: 'immich-edit/reject' } as TagRef
}));
const addTagToAsset = h.addTagToAsset;
const removeTagFromAsset = h.removeTagFromAsset;

vi.mock('$lib/api/tags', () => ({
  addTagToAsset: h.addTagToAsset,
  removeTagFromAsset: h.removeTagFromAsset
}));
vi.mock('$lib/api/assets', () => ({ updateAsset: h.updateAsset }));
vi.mock('$lib/stores/metadataConsent.svelte', () => ({
  metadataConsent: { gate: () => Promise.resolve(h.consent.value) }
}));
vi.mock('$lib/reject', async (orig) => {
  const actual = await orig<typeof import('./reject')>();
  return { ...actual, ensureRejectTag: () => Promise.resolve(h.rejectTag) };
});

import { toggleReject } from './cull';
import { browsing } from './stores/browsing.svelte';
import { isRejected } from './reject';

function asset(id: string, tags: TagRef[] = []): AssetSummary {
  return {
    id,
    originalFileName: id,
    type: 'IMAGE',
    fileCreatedAt: null,
    updatedAt: null,
    checksum: null,
    isFavorite: false,
    exifInfo: null,
    tags
  };
}

describe('toggleReject', () => {
  beforeEach(() => {
    h.consent.value = true;
    addTagToAsset.mockClear();
    removeTagFromAsset.mockClear();
    browsing.set([asset('a')]);
  });

  it('returns false for unknown asset', async () => {
    expect(await toggleReject('missing')).toBe(false);
  });

  it('adds the reject tag and patches browsing', async () => {
    expect(await toggleReject('a')).toBe(true);
    expect(addTagToAsset).toHaveBeenCalledWith('r', 'a');
    expect(isRejected(browsing.assets[0])).toBe(true);
  });

  it('removes the reject tag when already rejected', async () => {
    browsing.set([asset('a', [rejectTag])]);
    expect(await toggleReject('a')).toBe(true);
    expect(removeTagFromAsset).toHaveBeenCalledWith('r', 'a');
    expect(isRejected(browsing.assets[0])).toBe(false);
  });

  it('rolls back on api error', async () => {
    addTagToAsset.mockImplementationOnce(() => Promise.reject(new Error('boom')));
    expect(await toggleReject('a')).toBe(true);
    expect(isRejected(browsing.assets[0])).toBe(false);
  });

  it('returns false and makes no change when consent denied', async () => {
    h.consent.value = false;
    expect(await toggleReject('a')).toBe(false);
    expect(addTagToAsset).not.toHaveBeenCalled();
    expect(isRejected(browsing.assets[0])).toBe(false);
  });
});
