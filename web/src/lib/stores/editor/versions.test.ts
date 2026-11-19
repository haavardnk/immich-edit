import type { CopyRecord } from '#lib/api/copies.ts';
import type { EditHistoryEntry } from '#lib/api/edits.ts';
import type { Version } from '#lib/editor/versions.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { flushSync } from 'svelte';
import { beforeEach, describe, expect, it, vi, type Mock } from 'vitest';

const api = vi.hoisted(() => ({
  listCopies: vi.fn(),
  renameCopy: vi.fn(),
  deleteCopy: vi.fn(),
  listEditHistory: vi.fn()
}));

vi.mock('#lib/api/copies.ts', () => ({
  listCopies: api.listCopies,
  renameCopy: api.renameCopy,
  deleteCopy: api.deleteCopy
}));
vi.mock('#lib/api/edits.ts', () => ({ listEditHistory: api.listEditHistory }));

import { VersionStore } from '#lib/stores/editor/versions.svelte.ts';

const MASTER = '00000000-0000-0000-0000-000000000001';
const OTHER = '00000000-0000-0000-0000-000000000002';

function copy(index: number, name: string | null = null, source = MASTER): CopyRecord {
  return { id: `${source}_${index}`, source_asset_id: source, name, created_at: '' };
}

function history(action: string): EditHistoryEntry[] {
  return [
    {
      id: 1,
      manifest_hash: 'h',
      deleted: false,
      edits: null,
      created_at: '',
      action,
      undone: false
    }
  ];
}

describe('VersionStore', () => {
  let ctx: { assetId: string | null; compare: { versionRemoved: Mock<(id: string) => void> } };
  let store: VersionStore;

  function at(index: number): Version {
    const version = store.items[index];
    if (!version) throw new Error(`no version ${index}`);
    return version;
  }

  beforeEach(async () => {
    vi.clearAllMocks();
    toasts.items = [];
    ctx = { assetId: `${MASTER}_1`, compare: { versionRemoved: vi.fn() } };
    store = new VersionStore(ctx);
    api.listCopies.mockResolvedValue([copy(1, 'Mono'), copy(2)]);
    await store.load();
  });

  it('lists the original and the copies and marks the open one', () => {
    expect(store.items.map((version) => version.name)).toEqual(['Original', 'Mono', 'Copy 2']);
    expect(store.current?.id).toBe(`${MASTER}_1`);
    expect(store.copies).toHaveLength(2);
  });

  it('keeps the list across versions of the same photo and drops it for another photo', () => {
    ctx.assetId = MASTER;
    expect(store.copies).toHaveLength(2);
    expect(store.current?.name).toBe('Original');
    ctx.assetId = OTHER;
    expect(store.copies).toEqual([]);
    expect(store.items.map((version) => version.id)).toEqual([OTHER]);
  });

  it('loads once per photo and shares a load in flight', async () => {
    await Promise.all([store.load(), store.load()]);
    expect(api.listCopies).toHaveBeenCalledTimes(1);
    ctx.assetId = MASTER;
    await Promise.all([store.load(), store.load()]);
    expect(api.listCopies).toHaveBeenCalledTimes(2);
    store.reset();
    await store.load();
    expect(api.listCopies).toHaveBeenCalledTimes(3);
  });

  it('ignores a list that arrives after a reset', async () => {
    store.reset();
    let resolveFirst: (copies: CopyRecord[]) => void = () => {};
    api.listCopies.mockReturnValueOnce(new Promise((resolve) => (resolveFirst = resolve)));
    const first = store.load();
    store.reset();
    api.listCopies.mockResolvedValueOnce([copy(1, 'Fresh')]);
    await store.load();
    resolveFirst([copy(1, 'Stale')]);
    await first;
    expect(store.items.map((version) => version.name)).toEqual(['Original', 'Fresh']);
  });

  it('renames a copy, clears an empty name and patches the grid', async () => {
    const patch = vi.spyOn(browsing, 'patch');
    api.renameCopy.mockResolvedValueOnce(copy(2, 'Square crop'));
    expect(await store.rename(`${MASTER}_2`, '  Square crop ')).toBe(true);
    expect(api.renameCopy).toHaveBeenCalledWith(`${MASTER}_2`, 'Square crop');
    expect(store.items[2]?.name).toBe('Square crop');
    expect(patch).toHaveBeenCalledWith(`${MASTER}_2`, { copyLabel: 'Square crop' });

    api.renameCopy.mockResolvedValueOnce(copy(1));
    await store.rename(`${MASTER}_1`, '   ');
    expect(api.renameCopy).toHaveBeenLastCalledWith(`${MASTER}_1`, null);
    expect(store.items[1]?.name).toBe('Copy 1');
  });

  it('reports a failed rename and keeps the name', async () => {
    api.renameCopy.mockRejectedValueOnce(new Error('offline'));
    expect(await store.rename(`${MASTER}_1`, 'New')).toBe(false);
    expect(store.items[1]?.name).toBe('Mono');
    expect(toasts.items.at(-1)?.message).toContain('Could not rename the copy');
  });

  it('deletes a copy, removes it from the grid, ends a hold on it and tells compare', async () => {
    const remove = vi.spyOn(browsing, 'remove');
    ctx.assetId = MASTER;
    store.hold.start(at(2));
    api.deleteCopy.mockResolvedValueOnce(undefined);
    expect(await store.remove(`${MASTER}_2`)).toBe(true);
    expect(store.items.map((version) => version.name)).toEqual(['Original', 'Mono']);
    expect(remove).toHaveBeenCalledWith(`${MASTER}_2`);
    expect(store.hold.url).toBeNull();
    expect(ctx.compare.versionRemoved).toHaveBeenCalledWith(`${MASTER}_2`);
  });

  it('keeps a copy whose delete failed', async () => {
    api.deleteCopy.mockRejectedValueOnce(new Error('offline'));
    expect(await store.remove(`${MASTER}_2`)).toBe(false);
    expect(store.copies).toHaveLength(2);
    expect(ctx.compare.versionRemoved).not.toHaveBeenCalled();
    expect(toasts.items.at(-1)?.message).toContain('Could not delete the copy');
  });

  it('holds the persisted render of another version', () => {
    expect(store.hold.start(at(0))).toBe(true);
    expect(store.hold.url).toContain(`/api/assets/${MASTER}/preview?max=`);
    expect(store.hold.pill?.tail).toBe('· release to return to Mono');
    store.reset();
    expect(store.hold.url).toBeNull();
  });
});

describe('VersionSummaries', () => {
  let store: VersionStore;

  beforeEach(() => {
    vi.clearAllMocks();
    store = new VersionStore({ assetId: MASTER, compare: { versionRemoved: vi.fn() } });
    api.listEditHistory.mockImplementation((id: string) => Promise.resolve(history(`edit ${id}`)));
  });

  it('fetches each version once per open and refetches after a clear', async () => {
    store.summaries.load([MASTER, `${MASTER}_1`]);
    store.summaries.load([MASTER, `${MASTER}_1`]);
    await vi.waitFor(() => expect(store.summaries.get(`${MASTER}_1`)).toBeDefined());
    flushSync();
    expect(api.listEditHistory).toHaveBeenCalledTimes(2);
    expect(store.summaries.get(MASTER)?.[0]?.action).toBe(`edit ${MASTER}`);

    store.summaries.clear();
    expect(store.summaries.get(MASTER)).toBeUndefined();
    store.summaries.load([MASTER]);
    expect(api.listEditHistory).toHaveBeenCalledTimes(3);
  });

  it('drops answers that arrive after a clear', async () => {
    let resolve: (entries: EditHistoryEntry[]) => void = () => {};
    api.listEditHistory.mockReturnValueOnce(new Promise((done) => (resolve = done)));
    store.summaries.load([MASTER]);
    store.summaries.clear();
    resolve(history('late'));
    await Promise.resolve();
    expect(store.summaries.get(MASTER)).toBeUndefined();
  });

  it('leaves a failed version without a summary', async () => {
    api.listEditHistory.mockRejectedValueOnce(new Error('offline'));
    store.summaries.load([MASTER]);
    await Promise.resolve();
    await Promise.resolve();
    expect(store.summaries.get(MASTER)).toBeUndefined();
  });
});
