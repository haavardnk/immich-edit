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
vi.mock('$lib/managedTags', async (orig) => {
  const actual = await orig<typeof import('./managedTags')>();
  return {
    ...actual,
    ensureManagedTag: (value: string) =>
      Promise.resolve({ id: value.split('/').at(-1) ?? value, name: value, value })
  };
});

import { setLabel, toggleReject } from './cull';
import { browsing } from './stores/browsing.svelte';
import { isRejected } from './reject';
import { labelOf } from './labels';
import { labelMembers } from './stores/labels.svelte';

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

describe('setLabel', () => {
  const red: TagRef = { id: 'red', name: 'red', value: 'immich-edit/label/red' };

  beforeEach(() => {
    h.consent.value = true;
    addTagToAsset.mockClear();
    removeTagFromAsset.mockClear();
    for (const members of Object.values(labelMembers)) members.reset();
  });

  it('swaps one label for another in Immich and in the list', async () => {
    browsing.set([asset('a', [red])]);
    expect(await setLabel('a', 'blue')).toBe(true);
    expect(removeTagFromAsset).toHaveBeenCalledWith('red', 'a');
    expect(addTagToAsset).toHaveBeenCalledWith('blue', 'a');
    expect(labelOf(browsing.assets[0]!)).toBe('blue');
    expect(labelMembers.blue.ids.has('a')).toBe(true);
  });

  it('clears the label', async () => {
    browsing.set([asset('a', [red])]);
    labelMembers.red.add('a', red);
    expect(await setLabel('a', null)).toBe(true);
    expect(addTagToAsset).not.toHaveBeenCalled();
    expect(labelOf(browsing.assets[0]!)).toBeNull();
    expect(labelMembers.red.ids.has('a')).toBe(false);
  });

  it('restores the old label when Immich refuses', async () => {
    browsing.set([asset('a', [red])]);
    addTagToAsset.mockImplementationOnce(() => Promise.reject(new Error('boom')));
    expect(await setLabel('a', 'green')).toBe(true);
    expect(labelOf(browsing.assets[0]!)).toBe('red');
    expect(labelMembers.red.ids.has('a')).toBe(true);
    expect(labelMembers.green.ids.has('a')).toBe(false);
  });

  it('does nothing without consent', async () => {
    browsing.set([asset('a')]);
    h.consent.value = false;
    expect(await setLabel('a', 'red')).toBe(false);
    expect(labelOf(browsing.assets[0]!)).toBeNull();
  });
});

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
    expect(isRejected(browsing.assets[0]!)).toBe(true);
  });

  it('removes the reject tag when already rejected', async () => {
    browsing.set([asset('a', [rejectTag])]);
    expect(await toggleReject('a')).toBe(true);
    expect(removeTagFromAsset).toHaveBeenCalledWith('r', 'a');
    expect(isRejected(browsing.assets[0]!)).toBe(false);
  });

  it('rolls back on api error', async () => {
    addTagToAsset.mockImplementationOnce(() => Promise.reject(new Error('boom')));
    expect(await toggleReject('a')).toBe(true);
    expect(isRejected(browsing.assets[0]!)).toBe(false);
  });

  it('returns false and makes no change when consent denied', async () => {
    h.consent.value = false;
    expect(await toggleReject('a')).toBe(false);
    expect(addTagToAsset).not.toHaveBeenCalled();
    expect(isRejected(browsing.assets[0]!)).toBe(false);
  });
});
