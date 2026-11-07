import { describe, it, expect, beforeEach, vi } from 'vitest';
import type { AssetSummary } from '#lib/types/album.ts';
import type { TagRef } from '#lib/types/asset.ts';

const rejectTag: TagRef = { id: 'r', name: 'reject', value: 'immich-edit/reject' };
const h = vi.hoisted(() => ({
  addTagToAsset: vi.fn(() => Promise.resolve()),
  removeTagFromAsset: vi.fn(() => Promise.resolve()),
  updateAsset: vi.fn(() => Promise.resolve({} as never)),
  consent: { value: true },
  rejectTag: { id: 'r', name: 'reject', value: 'immich-edit/reject' } as TagRef | null
}));
const addTagToAsset = h.addTagToAsset;
const removeTagFromAsset = h.removeTagFromAsset;

vi.mock('#lib/api/tags.ts', () => ({
  addTagToAsset: h.addTagToAsset,
  removeTagFromAsset: h.removeTagFromAsset
}));
vi.mock('#lib/api/assets.ts', () => ({ updateAsset: h.updateAsset }));
vi.mock('#lib/stores/metadataConsent.svelte.ts', () => ({
  metadataConsent: { gate: () => Promise.resolve(h.consent.value) }
}));
vi.mock('#lib/browse/reject.ts', async (orig) => {
  const actual = await orig<typeof import('./reject')>();
  return { ...actual, ensureRejectTag: () => Promise.resolve(h.rejectTag) };
});
vi.mock('#lib/browse/managedTags.ts', async (orig) => {
  const actual = await orig<typeof import('./managedTags')>();
  return {
    ...actual,
    ensureManagedTag: (value: string) =>
      Promise.resolve({ id: value.split('/').at(-1) ?? value, name: value, value })
  };
});

import {
  browseTarget,
  clearFlags,
  setFavorite,
  setLabel,
  setDescription,
  setRating,
  toggleFavorite,
  toggleReject,
  type MetaKind,
  type MetaTarget
} from './assetMeta';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { session } from '#lib/stores/session.svelte.ts';
import { isRejected } from './reject';
import { labelOf } from '#lib/stores/labels.ts';
import { labelMembers } from '#lib/stores/labels.svelte.ts';
import { rejected } from '#lib/stores/rejected.svelte.ts';

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

function recordingTarget(id: string): { target: MetaTarget; failures: [MetaKind, unknown][] } {
  const failures: [MetaKind, unknown][] = [];
  return { target: browseTarget(id, (kind, error) => failures.push([kind, error])), failures };
}

beforeEach(() => {
  h.consent.value = true;
  h.rejectTag = rejectTag;
  addTagToAsset.mockClear();
  removeTagFromAsset.mockClear();
  h.updateAsset.mockClear();
  session.clear();
});

describe('setRating', () => {
  it('shows the rating before Immich answers and keeps it', async () => {
    browsing.set([asset('a')]);
    const pending = setRating(browseTarget('a'), 4);
    await vi.waitFor(() => expect(h.updateAsset).toHaveBeenCalledWith('a', { rating: 4 }));
    expect(browsing.assets[0]?.exifInfo?.rating).toBe(4);
    expect(await pending).toBe('saved');
    expect(browsing.assets[0]?.exifInfo?.rating).toBe(4);
  });

  it('restores the old rating and reports the failure', async () => {
    browsing.set([{ ...asset('a'), exifInfo: { ...asset('a').exifInfo, rating: 2 } as never }]);
    const boom = new Error('boom');
    h.updateAsset.mockImplementationOnce(() => Promise.reject(boom));
    const { target, failures } = recordingTarget('a');
    expect(await setRating(target, 5)).toBe('failed');
    expect(browsing.assets[0]?.exifInfo?.rating).toBe(2);
    expect(failures).toEqual([['rating', boom]]);
  });

  it('keeps a newer rating when an older write fails', async () => {
    browsing.set([asset('a')]);
    let refuse: (error: Error) => void = () => {};
    h.updateAsset.mockImplementationOnce(
      () => new Promise<never>((_, reject) => (refuse = reject))
    );
    const { target, failures } = recordingTarget('a');
    const older = setRating(target, 3);
    await vi.waitFor(() => expect(h.updateAsset).toHaveBeenCalledTimes(1));
    const newer = setRating(target, 4);
    await vi.waitFor(() => expect(browsing.assets[0]?.exifInfo?.rating).toBe(4));
    refuse(new Error('boom'));
    expect(await Promise.all([older, newer])).toEqual(['saved', 'saved']);
    expect(h.updateAsset).toHaveBeenLastCalledWith('a', { rating: 4 });
    expect(browsing.assets[0]?.exifInfo?.rating).toBe(4);
    expect(failures).toEqual([]);
  });

  it('writes into the listed photo in place', async () => {
    const element = asset('a');
    browsing.set([element]);
    await setRating(browseTarget('a'), 3);
    expect(element.exifInfo?.rating).toBe(3);
  });
});

describe('setDescription', () => {
  it('writes a changed description and shows it at once', async () => {
    browsing.set([asset('a')]);
    expect(await setDescription(browseTarget('a'), 'Fløyen at dusk')).toBe('saved');
    expect(h.updateAsset).toHaveBeenCalledWith('a', { description: 'Fløyen at dusk' });
    expect(browsing.assets[0]?.exifInfo?.description).toBe('Fløyen at dusk');
    expect(browsing.assets[0]?.exifInfo?.rating).toBeNull();
  });

  it('skips an unchanged description and a photo shared by someone else', async () => {
    browsing.set([
      { ...asset('a'), exifInfo: { description: 'Same' } as never },
      { ...asset('b'), ownerId: 'someone-else' }
    ]);
    expect(await setDescription(browseTarget('a'), 'Same')).toBe('skipped');
    expect(await setDescription(browseTarget('b'), 'New')).toBe('skipped');
    expect(h.updateAsset).not.toHaveBeenCalled();
  });

  it('restores the old description when Immich refuses', async () => {
    browsing.set([{ ...asset('a'), exifInfo: { description: 'Old' } as never }]);
    const boom = new Error('boom');
    h.updateAsset.mockImplementationOnce(() => Promise.reject(boom));
    const { target, failures } = recordingTarget('a');
    expect(await setDescription(target, 'New')).toBe('failed');
    expect(browsing.assets[0]?.exifInfo?.description).toBe('Old');
    expect(failures).toEqual([['description', boom]]);
  });
});

describe('favorite', () => {
  it('toggles from the current state', async () => {
    browsing.set([{ ...asset('a'), isFavorite: true }]);
    expect(await toggleFavorite(browseTarget('a'))).toBe('saved');
    expect(h.updateAsset).toHaveBeenCalledWith('a', { isFavorite: false });
    expect(browsing.assets[0]?.isFavorite).toBe(false);
  });

  it('rolls back when Immich refuses', async () => {
    browsing.set([asset('a')]);
    h.updateAsset.mockImplementationOnce(() => Promise.reject(new Error('boom')));
    expect(await setFavorite(recordingTarget('a').target, true)).toBe('failed');
    expect(browsing.assets[0]?.isFavorite).toBe(false);
  });

  it('skips a photo that changed while consent was asked', async () => {
    let current: AssetSummary | null = asset('a');
    const target: MetaTarget = {
      id: 'a',
      read: () => {
        const seen = current;
        current = null;
        return seen;
      },
      patch: vi.fn(),
      fail: vi.fn()
    };
    expect(await setFavorite(target, true)).toBe('skipped');
    expect(h.updateAsset).not.toHaveBeenCalled();
    expect(target.patch).not.toHaveBeenCalled();
  });
});

describe('setLabel', () => {
  const red: TagRef = { id: 'red', name: 'red', value: 'immich-edit/label/red' };

  beforeEach(() => {
    for (const members of Object.values(labelMembers)) members.reset();
  });

  it('swaps one label for another in Immich and in the list', async () => {
    browsing.set([asset('a', [red])]);
    expect(await setLabel(browseTarget('a'), 'blue')).toBe('saved');
    expect(removeTagFromAsset).toHaveBeenCalledWith('red', 'a');
    expect(addTagToAsset).toHaveBeenCalledWith('blue', 'a');
    expect(labelOf(browsing.assets[0]!)).toBe('blue');
    expect(labelMembers.blue.ids.has('a')).toBe(true);
  });

  it('clears the label', async () => {
    browsing.set([asset('a', [red])]);
    labelMembers.red.add('a', red);
    expect(await setLabel(browseTarget('a'), null)).toBe('saved');
    expect(addTagToAsset).not.toHaveBeenCalled();
    expect(labelOf(browsing.assets[0]!)).toBeNull();
    expect(labelMembers.red.ids.has('a')).toBe(false);
  });

  it('restores the old label when Immich refuses', async () => {
    browsing.set([asset('a', [red])]);
    labelMembers.red.add('a', red);
    addTagToAsset.mockImplementationOnce(() => Promise.reject(new Error('boom')));
    expect(await setLabel(recordingTarget('a').target, 'green')).toBe('failed');
    expect(labelOf(browsing.assets[0]!)).toBe('red');
    expect(labelMembers.red.ids.has('a')).toBe(true);
    expect(labelMembers.green.ids.has('a')).toBe(false);
  });

  it('does nothing without consent', async () => {
    browsing.set([asset('a')]);
    h.consent.value = false;
    expect(await setLabel(browseTarget('a'), 'red')).toBe('skipped');
    expect(labelOf(browsing.assets[0]!)).toBeNull();
  });

  it('swaps out a label that was still being written', async () => {
    browsing.set([asset('a')]);
    let land: () => void = () => {};
    addTagToAsset.mockImplementationOnce(() => new Promise<void>((resolve) => (land = resolve)));
    const first = setLabel(browseTarget('a'), 'red');
    await vi.waitFor(() => expect(addTagToAsset).toHaveBeenCalledWith('red', 'a'));
    const second = setLabel(browseTarget('a'), 'blue');
    await vi.waitFor(() => expect(labelOf(browsing.assets[0]!)).toBe('blue'));
    land();
    expect(await Promise.all([first, second])).toEqual(['saved', 'saved']);
    expect(removeTagFromAsset).toHaveBeenCalledWith('red', 'a');
    expect(addTagToAsset).toHaveBeenLastCalledWith('blue', 'a');
    expect(labelOf(browsing.assets[0]!)).toBe('blue');
  });

  it('keeps the old label when Immich refuses the label it replaced', async () => {
    browsing.set([asset('a', [red])]);
    let refuse: () => void = () => {};
    removeTagFromAsset.mockImplementationOnce(
      () => new Promise<void>((_, reject) => (refuse = () => reject(new Error('boom'))))
    );
    const first = setLabel(recordingTarget('a').target, 'green');
    await vi.waitFor(() => expect(removeTagFromAsset).toHaveBeenCalledTimes(1));
    const second = setLabel(recordingTarget('a').target, 'blue');
    await vi.waitFor(() => expect(labelOf(browsing.assets[0]!)).toBe('blue'));
    refuse();
    await Promise.all([first, second]);
    expect(removeTagFromAsset).toHaveBeenLastCalledWith('red', 'a');
    expect(removeTagFromAsset).toHaveBeenCalledTimes(2);
    expect(labelOf(browsing.assets[0]!)).toBe('blue');
  });
});

describe('assets owned by another Immich user', () => {
  beforeEach(() => {
    session.set({ id: 'me', email: 'me@x', name: 'Me', is_admin: false, auth_kind: 'password' });
  });

  it.each([
    ['rating', () => setRating(browseTarget('a'), 3)],
    ['favorite', () => toggleFavorite(browseTarget('a'))],
    ['label', () => setLabel(browseTarget('a'), 'red')],
    ['reject', () => toggleReject(browseTarget('a'))]
  ])('leaves %s untouched', async (_, write) => {
    browsing.set([{ ...asset('a'), ownerId: 'partner' }]);
    expect(await write()).toBe('skipped');
    expect(h.updateAsset).not.toHaveBeenCalled();
    expect(addTagToAsset).not.toHaveBeenCalled();
    expect(browsing.assets[0]?.tags).toEqual([]);
  });

  it('still writes own assets', async () => {
    browsing.set([{ ...asset('a'), ownerId: 'me' }]);
    expect(await toggleFavorite(browseTarget('a'))).toBe('saved');
    expect(h.updateAsset).toHaveBeenCalledWith('a', { isFavorite: true });
  });
});

describe('toggleReject', () => {
  beforeEach(() => {
    browsing.set([asset('a')]);
    rejected.reset();
  });

  it('skips an unknown asset', async () => {
    expect(await toggleReject(browseTarget('missing'))).toBe('skipped');
  });

  it('adds the reject tag and patches browsing', async () => {
    expect(await toggleReject(browseTarget('a'))).toBe('saved');
    expect(addTagToAsset).toHaveBeenCalledWith('r', 'a');
    expect(isRejected(browsing.assets[0]!)).toBe(true);
    expect(rejected.ids.has('a')).toBe(true);
  });

  it('removes the reject tag when already rejected', async () => {
    browsing.set([asset('a', [rejectTag])]);
    expect(await toggleReject(browseTarget('a'))).toBe('saved');
    expect(removeTagFromAsset).toHaveBeenCalledWith('r', 'a');
    expect(isRejected(browsing.assets[0]!)).toBe(false);
  });

  it('rolls back on api error', async () => {
    addTagToAsset.mockImplementationOnce(() => Promise.reject(new Error('boom')));
    expect(await toggleReject(recordingTarget('a').target)).toBe('failed');
    expect(isRejected(browsing.assets[0]!)).toBe(false);
    expect(rejected.ids.has('a')).toBe(false);
  });

  it('reports a reject tag it cannot create and writes nothing', async () => {
    h.rejectTag = null;
    const { target, failures } = recordingTarget('a');
    expect(await toggleReject(target)).toBe('skipped');
    expect(failures).toEqual([['reject', 'could not create tag']]);
    expect(addTagToAsset).not.toHaveBeenCalled();
  });

  it('skips and makes no change when consent denied', async () => {
    h.consent.value = false;
    expect(await toggleReject(browseTarget('a'))).toBe('skipped');
    expect(addTagToAsset).not.toHaveBeenCalled();
    expect(isRejected(browsing.assets[0]!)).toBe(false);
  });
});

describe('clearFlags', () => {
  it('removes the favorite and the reject', async () => {
    browsing.set([{ ...asset('a', [rejectTag]), isFavorite: true }]);
    expect(await clearFlags(browseTarget('a'))).toBe('saved');
    expect(h.updateAsset).toHaveBeenCalledWith('a', { isFavorite: false });
    expect(removeTagFromAsset).toHaveBeenCalledWith('r', 'a');
    expect(browsing.assets[0]?.isFavorite).toBe(false);
    expect(isRejected(browsing.assets[0]!)).toBe(false);
  });

  it('skips a photo with no flags', async () => {
    browsing.set([asset('a')]);
    expect(await clearFlags(browseTarget('a'))).toBe('skipped');
    expect(h.updateAsset).not.toHaveBeenCalled();
  });

  it('reports a failure from either write', async () => {
    browsing.set([{ ...asset('a', [rejectTag]), isFavorite: true }]);
    removeTagFromAsset.mockImplementationOnce(() => Promise.reject(new Error('boom')));
    expect(await clearFlags(recordingTarget('a').target)).toBe('failed');
    expect(browsing.assets[0]?.isFavorite).toBe(false);
    expect(isRejected(browsing.assets[0]!)).toBe(true);
  });
});
