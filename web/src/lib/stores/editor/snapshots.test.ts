import { ConflictError } from '#lib/api/client.ts';
import type { EditSnapshot } from '#lib/api/snapshots.ts';
import { editsToManifest } from '#lib/edits/manifest.ts';
import type { AssetDetail } from '#lib/types/asset.ts';
import { neutralEdits, type EditRecord, type Edits } from '#lib/types/edits.ts';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  getAsset: vi.fn(),
  getEdits: vi.fn(),
  putEdits: vi.fn(),
  listSnapshots: vi.fn(),
  createSnapshot: vi.fn(),
  updateSnapshot: vi.fn(),
  deleteSnapshot: vi.fn(),
  restoreSnapshot: vi.fn()
}));

vi.mock('#lib/api/assets.ts', () => ({ getAsset: mocks.getAsset }));
vi.mock('#lib/api/edits.ts', () => ({
  getEdits: mocks.getEdits,
  putEdits: mocks.putEdits,
  deleteEdits: vi.fn(() => Promise.resolve()),
  restoreEdits: vi.fn(),
  listEditedAssets: vi.fn(),
  autoEdits: vi.fn()
}));
vi.mock('#lib/api/snapshots.ts', () => ({
  listSnapshots: mocks.listSnapshots,
  createSnapshot: mocks.createSnapshot,
  updateSnapshot: mocks.updateSnapshot,
  deleteSnapshot: mocks.deleteSnapshot,
  restoreSnapshot: mocks.restoreSnapshot
}));
vi.mock('#lib/api/lensProfile.ts', () => ({ getLensProfile: vi.fn(() => Promise.resolve(null)) }));
vi.mock('#lib/api/preview.ts', async (original) => ({
  ...(await original<typeof import('#lib/api/preview.ts')>()),
  livePreview: vi.fn(() => new Promise<never>(() => undefined))
}));

import { editor } from '#lib/stores/editor.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';

function asset(): AssetDetail {
  return {
    id: 'asset-1',
    originalFileName: 'photo.raw',
    type: 'IMAGE',
    originalMimeType: 'image/x-raw',
    fileCreatedAt: null,
    updatedAt: null,
    checksum: null,
    isFavorite: false,
    exifInfo: null,
    tags: []
  };
}

function editsAt(exposure: number): Edits {
  const edits = neutralEdits();
  edits.basic.exposure_ev = exposure;
  return edits;
}

function record(hash: string, exposure: number): EditRecord {
  return {
    schema_version: 1,
    asset_id: 'asset-1',
    renderer_version: 'test',
    manifest: editsToManifest(editsAt(exposure)),
    hash,
    updated_at: '2026-09-01T00:00:00Z',
    immich_updated_at: null,
    immich_checksum: null
  };
}

function snapshot(id: string, name: string, hash: string, exposure: number): EditSnapshot {
  return {
    id,
    name,
    manifest_hash: hash,
    edits: editsAt(exposure),
    created_at: '2026-09-01T00:00:00Z',
    updated_at: '2026-09-01T00:00:00Z'
  };
}

function deferred<T>(): { promise: Promise<T>; resolve: (value: T) => void } {
  let resolve = (_value: T): void => undefined;
  const promise = new Promise<T>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}

const WARM = snapshot('snap-warm', 'Warm v1', 'hash-warm', 0.7);

describe('editor snapshots', () => {
  beforeEach(async () => {
    editor.unload();
    for (const mock of Object.values(mocks)) mock.mockReset();
    toasts.items = [];
    mocks.getAsset.mockResolvedValue(asset());
    mocks.getEdits.mockResolvedValue(record('hash-0', 0));
    mocks.listSnapshots.mockResolvedValue([WARM, snapshot('snap-1', 'Snapshot 1', 'h1', 0)]);
    await editor.load('asset-1');
    await vi.waitFor(() => expect(editor.snapshots.items).toHaveLength(2));
  });

  it('loads the snapshots of the editor asset and clears them on unload', () => {
    expect(mocks.listSnapshots).toHaveBeenCalledWith('asset-1');
    editor.unload();
    expect(editor.snapshots.items).toEqual([]);
  });

  it('names a new snapshot after the first free number and posts the current edits', async () => {
    mocks.createSnapshot.mockImplementation((_asset: string, name: string, edits: Edits) =>
      Promise.resolve({ ...snapshot('snap-2', name, 'h2', 0), edits })
    );
    editor.edits.basic.exposure_ev = 1.5;

    const created = await editor.snapshots.create();

    expect(mocks.createSnapshot).toHaveBeenCalledTimes(1);
    const [assetId, name, edits] = mocks.createSnapshot.mock.calls[0] as [string, string, Edits];
    expect(assetId).toBe('asset-1');
    expect(name).toBe('Snapshot 2');
    expect(edits.basic.exposure_ev).toBe(1.5);
    expect(created?.name).toBe('Snapshot 2');
    expect(editor.snapshots.items.map((item) => item.id)).toEqual([
      'snap-2',
      'snap-warm',
      'snap-1'
    ]);
  });

  it('restores after pending saves and pushes the restore onto the undo stack', async () => {
    const save = deferred<EditRecord>();
    mocks.putEdits.mockImplementationOnce(() => save.promise);
    mocks.putEdits.mockResolvedValue(record('hash-undo', 1));
    mocks.restoreSnapshot.mockResolvedValue(record('hash-warm', 0.7));

    editor.edits.basic.exposure_ev = 1;
    const saving = editor.onCommit('Exposure');
    const restoring = editor.snapshots.restore('snap-warm');

    await vi.waitFor(() => expect(mocks.putEdits).toHaveBeenCalledTimes(1));
    expect(mocks.restoreSnapshot).not.toHaveBeenCalled();

    save.resolve(record('hash-1', 1));
    await saving;
    await expect(restoring).resolves.toBe(true);

    expect(mocks.restoreSnapshot).toHaveBeenCalledWith('asset-1', 'snap-warm', 'hash-1');
    expect(editor.edits.basic.exposure_ev).toBe(0.7);
    expect(editor.savedHash).toBe('hash-warm');
    expect(editor.snapshots.restoredId).toBe('snap-warm');

    const toast = toasts.items.at(-1);
    expect(toast?.message).toBe('Restored Warm v1');
    expect(toast?.action?.label).toBe('Undo');

    toast?.action?.run();
    expect(editor.edits.basic.exposure_ev).toBe(1);
    await vi.waitFor(() => expect(editor.savedHash).toBe('hash-undo'));
    expect(editor.snapshots.restoredId).toBeNull();
  });

  it('keeps local edits when the restore conflicts', async () => {
    mocks.restoreSnapshot.mockRejectedValue(
      new ConflictError('conflict', record('hash-remote', 3))
    );
    editor.edits.basic.exposure_ev = 2;

    await expect(editor.snapshots.restore('snap-warm')).resolves.toBe(false);

    expect(editor.edits.basic.exposure_ev).toBe(2);
    expect(editor.savedHash).toBe('hash-remote');
    expect(editor.saveError).toContain('changed elsewhere');
    expect(editor.snapshots.restoredId).toBeNull();
    expect(editor.canUndo).toBe(false);
  });

  it('keeps an edit queued behind a restore in the view and on the server', async () => {
    const restore = deferred<EditRecord>();
    mocks.restoreSnapshot.mockImplementationOnce(() => restore.promise);
    mocks.putEdits.mockResolvedValue(record('hash-after', 2));

    const restoring = editor.snapshots.restore('snap-warm');
    await vi.waitFor(() => expect(mocks.restoreSnapshot).toHaveBeenCalledTimes(1));
    editor.edits.basic.exposure_ev = 2;
    const saving = editor.onCommit('Exposure');

    restore.resolve(record('hash-warm', 0.7));
    await Promise.all([restoring, saving]);

    expect(editor.edits.basic.exposure_ev).toBe(2);
    const [, sent, baseHash] = mocks.putEdits.mock.calls[0] as [string, Edits, string];
    expect(sent.basic.exposure_ev).toBe(2);
    expect(baseHash).toBe('hash-warm');
    expect(editor.savedHash).toBe('hash-after');
  });

  it('ignores the toast undo once a later edit has been made', async () => {
    mocks.restoreSnapshot.mockResolvedValue(record('hash-warm', 0.7));
    mocks.putEdits.mockResolvedValue(record('hash-after', 2));
    await editor.snapshots.restore('snap-warm');
    const toast = toasts.items.at(-1);

    editor.edits.basic.exposure_ev = 2;
    await editor.onCommit('Exposure');
    toast?.action?.run();

    expect(editor.edits.basic.exposure_ev).toBe(2);
    expect(mocks.putEdits).toHaveBeenCalledTimes(1);
  });

  it('updates a snapshot with the current edits and renames it', async () => {
    mocks.updateSnapshot.mockImplementation(
      (_asset: string, id: string, change: { name?: string; edits?: Edits }) =>
        Promise.resolve({
          ...WARM,
          id,
          name: change.name ?? WARM.name,
          manifest_hash: change.edits ? 'hash-new' : WARM.manifest_hash,
          edits: change.edits ?? WARM.edits
        })
    );
    editor.edits.basic.exposure_ev = 2.5;

    await editor.snapshots.updateWithCurrent('snap-warm');
    const [, , change] = mocks.updateSnapshot.mock.calls[0] as [string, string, { edits: Edits }];
    expect(change.edits.basic.exposure_ev).toBe(2.5);
    expect(editor.snapshots.items[0]?.manifest_hash).toBe('hash-new');

    await editor.snapshots.rename('snap-warm', '  Cool v2 ');
    expect(mocks.updateSnapshot).toHaveBeenLastCalledWith('asset-1', 'snap-warm', {
      name: 'Cool v2'
    });
    expect(editor.snapshots.items[0]?.name).toBe('Cool v2');
  });

  it('removes a deleted snapshot', async () => {
    mocks.deleteSnapshot.mockResolvedValue(undefined);
    await editor.snapshots.remove('snap-1');
    expect(mocks.deleteSnapshot).toHaveBeenCalledWith('asset-1', 'snap-1');
    expect(editor.snapshots.items.map((item) => item.id)).toEqual(['snap-warm']);
  });
});
