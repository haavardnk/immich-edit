import type { TagSummary } from '#lib/api/tags.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import type { TagRef } from '#lib/types/asset.ts';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const h = vi.hoisted(() => ({
  addTagToAsset: vi.fn((_tagId: string, _assetId: string) => Promise.resolve()),
  removeTagFromAsset: vi.fn((_tagId: string, _assetId: string) => Promise.resolve()),
  upsertTags: vi.fn((_values: string[]): Promise<TagSummary[]> => Promise.resolve([])),
  consent: { value: true }
}));

vi.mock('#lib/api/tags.ts', () => ({
  addTagToAsset: h.addTagToAsset,
  removeTagFromAsset: h.removeTagFromAsset,
  upsertTags: h.upsertTags
}));
vi.mock('#lib/api/assets.ts', () => ({ updateAsset: vi.fn(() => Promise.resolve()) }));
vi.mock('#lib/stores/metadataConsent.svelte.ts', () => ({
  metadataConsent: { gate: () => Promise.resolve(h.consent.value) }
}));
vi.mock('#lib/browse/reject.ts', async (orig) => {
  const actual = await orig<typeof import('#lib/browse/reject.ts')>();
  return {
    ...actual,
    ensureRejectTag: () => Promise.resolve({ id: 'r', name: 'reject', value: 'immich-edit/reject' })
  };
});

import {
  browseTarget,
  toggleReject,
  type MetaKind,
  type MetaTarget
} from '#lib/browse/assetMeta.ts';
import { addTag, createAndAddTag, removeTag } from '#lib/browse/tagOps.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { rejected } from '#lib/stores/rejected.svelte.ts';
import { session } from '#lib/stores/session.svelte.ts';

const sunset: TagRef = { id: 't1', name: 'sunset', value: 'sunset' };
const beach: TagRef = { id: 't2', name: 'beach', value: 'beach' };

function asset(tags: TagRef[] = []): AssetSummary {
  return {
    id: 'a',
    originalFileName: 'a',
    type: 'IMAGE',
    fileCreatedAt: null,
    updatedAt: null,
    checksum: null,
    isFavorite: false,
    exifInfo: null,
    tags
  };
}

function recordingTarget(): { target: MetaTarget; failures: [MetaKind, unknown][] } {
  const failures: [MetaKind, unknown][] = [];
  return { target: browseTarget('a', (kind, error) => failures.push([kind, error])), failures };
}

function tagIds(): string[] {
  return (browsing.find('a')?.tags ?? []).map((t) => t.id);
}

function held(): { promise: Promise<void>; refuse: () => void } {
  let refuse: () => void = () => {};
  const promise = new Promise<void>((_, reject) => {
    refuse = () => reject(new Error('boom'));
  });
  return { promise, refuse };
}

beforeEach(() => {
  h.consent.value = true;
  h.addTagToAsset.mockReset();
  h.addTagToAsset.mockResolvedValue();
  h.removeTagFromAsset.mockReset();
  h.removeTagFromAsset.mockResolvedValue();
  h.upsertTags.mockReset();
  h.upsertTags.mockResolvedValue([]);
  session.clear();
  rejected.reset();
  browsing.set([asset()]);
});

describe('addTag', () => {
  it('adds the tag in Immich and in the list', async () => {
    expect(await addTag(browseTarget('a'), sunset)).toBe('saved');
    expect(h.addTagToAsset).toHaveBeenCalledWith('t1', 'a');
    expect(tagIds()).toEqual(['t1']);
  });

  it('writes a tag the photo already has without duplicating it', async () => {
    browsing.set([asset([sunset])]);
    expect(await addTag(browseTarget('a'), sunset)).toBe('saved');
    expect(h.addTagToAsset).toHaveBeenCalledWith('t1', 'a');
    expect(tagIds()).toEqual(['t1']);
  });

  it('takes the tag back off when Immich refuses', async () => {
    h.addTagToAsset.mockRejectedValueOnce(new Error('boom'));
    const { target, failures } = recordingTarget();
    expect(await addTag(target, sunset)).toBe('failed');
    expect(tagIds()).toEqual([]);
    expect(failures.map(([kind]) => kind)).toEqual(['tag']);
  });

  it('writes nothing without consent', async () => {
    h.consent.value = false;
    expect(await addTag(browseTarget('a'), sunset)).toBe('skipped');
    expect(h.addTagToAsset).not.toHaveBeenCalled();
    expect(tagIds()).toEqual([]);
  });
});

describe('removeTag', () => {
  it('removes the tag in Immich and in the list', async () => {
    browsing.set([asset([sunset, beach])]);
    expect(await removeTag(browseTarget('a'), 't1')).toBe('saved');
    expect(h.removeTagFromAsset).toHaveBeenCalledWith('t1', 'a');
    expect(tagIds()).toEqual(['t2']);
  });

  it('puts the tag back when Immich refuses', async () => {
    browsing.set([asset([sunset, beach])]);
    h.removeTagFromAsset.mockRejectedValueOnce(new Error('boom'));
    expect(await removeTag(recordingTarget().target, 't1')).toBe('failed');
    expect(tagIds().toSorted()).toEqual(['t1', 't2']);
  });
});

describe('createAndAddTag', () => {
  it('creates the tag and adds it', async () => {
    h.upsertTags.mockResolvedValueOnce([{ ...sunset, createdAt: null }]);
    expect(await createAndAddTag(browseTarget('a'), 'sunset')).toEqual(
      expect.objectContaining(sunset)
    );
    expect(h.upsertTags).toHaveBeenCalledWith(['sunset']);
    expect(tagIds()).toEqual(['t1']);
  });

  it('reports a tag Immich cannot create', async () => {
    h.upsertTags.mockRejectedValueOnce(new Error('boom'));
    const { target, failures } = recordingTarget();
    expect(await createAndAddTag(target, 'sunset')).toBeNull();
    expect(failures.map(([kind]) => kind)).toEqual(['tag']);
    expect(h.addTagToAsset).not.toHaveBeenCalled();
  });

  it('returns nothing when the add fails', async () => {
    h.upsertTags.mockResolvedValueOnce([{ ...sunset, createdAt: null }]);
    h.addTagToAsset.mockRejectedValueOnce(new Error('boom'));
    expect(await createAndAddTag(recordingTarget().target, 'sunset')).toBeNull();
    expect(tagIds()).toEqual([]);
  });

  it('creates nothing without consent', async () => {
    h.consent.value = false;
    expect(await createAndAddTag(browseTarget('a'), 'sunset')).toBeNull();
    expect(h.upsertTags).not.toHaveBeenCalled();
  });
});

describe('tag writes beside a reject', () => {
  it('keep the reject when a pending tag add fails', async () => {
    const add = held();
    h.addTagToAsset.mockImplementationOnce(() => add.promise);
    const tagging = addTag(recordingTarget().target, sunset);
    await vi.waitFor(() => expect(h.addTagToAsset).toHaveBeenCalledTimes(1));
    expect(await toggleReject(browseTarget('a'))).toBe('saved');
    add.refuse();
    expect(await tagging).toBe('failed');
    expect(tagIds()).toEqual(['r']);
    expect(rejected.ids.has('a')).toBe(true);
  });

  it('keep the tag when a pending reject fails', async () => {
    const reject = held();
    h.addTagToAsset.mockImplementationOnce(() => reject.promise);
    const rejecting = toggleReject(recordingTarget().target);
    await vi.waitFor(() => expect(h.addTagToAsset).toHaveBeenCalledTimes(1));
    expect(await addTag(browseTarget('a'), sunset)).toBe('saved');
    reject.refuse();
    expect(await rejecting).toBe('failed');
    expect(tagIds()).toEqual(['t1']);
  });
});
