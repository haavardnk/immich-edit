import { beforeEach, describe, expect, it, vi } from 'vitest';
import { editsToManifest } from '#lib/edits/manifest.ts';
import type { AssetDetail } from '#lib/types/asset.ts';
import { neutralEdits, type EditRecord, type Edits } from '#lib/types/edits.ts';

const mocks = vi.hoisted(() => ({
  getAsset: vi.fn(),
  getEdits: vi.fn(),
  putEdits: vi.fn(),
  deleteEdits: vi.fn(() => Promise.resolve()),
  restoreEdits: vi.fn(),
  listEditHistory: vi.fn(() => Promise.resolve([])),
  getLensProfile: vi.fn(() => Promise.resolve(null)),
  livePreview: vi.fn(() => new Promise<never>(() => undefined))
}));

vi.mock('#lib/api/assets.ts', () => ({ getAsset: mocks.getAsset }));
vi.mock('#lib/api/edits.ts', () => ({
  getEdits: mocks.getEdits,
  putEdits: mocks.putEdits,
  deleteEdits: mocks.deleteEdits,
  restoreEdits: mocks.restoreEdits,
  listEditHistory: mocks.listEditHistory,
  autoEdits: vi.fn()
}));
vi.mock('#lib/api/lensProfile.ts', () => ({ getLensProfile: mocks.getLensProfile }));
vi.mock('#lib/api/preview.ts', async (original) => ({
  ...(await original<typeof import('#lib/api/preview.ts')>()),
  livePreview: mocks.livePreview
}));

import { editor } from '#lib/stores/editor.svelte.ts';
import { EditHistory } from '#lib/stores/editor/history.svelte.ts';
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

function deferred<T>(): {
  promise: Promise<T>;
  resolve: (value: T) => void;
  reject: (error: unknown) => void;
} {
  let resolve = (_value: T): void => undefined;
  let reject = (_error: unknown): void => undefined;
  const promise = new Promise<T>((done, fail) => {
    resolve = done;
    reject = fail;
  });
  return { promise, resolve, reject };
}

function jump(id: number, exposure: number, from: number | null = null): Promise<boolean> {
  return editor.restoreHistoryEntry({ id, edits: editsAt(exposure), from, label: `Step ${id}` });
}

describe('editor history', () => {
  beforeEach(async () => {
    editor.unload();
    mocks.putEdits.mockReset();
    mocks.restoreEdits.mockReset();
    toasts.items = [];
    mocks.putEdits.mockImplementation((_id: string, _edits: unknown, _hash: string) =>
      Promise.resolve(record('hash-saved', 0))
    );
    mocks.getAsset.mockResolvedValue(asset());
    mocks.getEdits.mockResolvedValue(record('hash-0', 0));
    await editor.load('asset-1');
  });

  it('starts with the loaded state as the only entry', () => {
    expect(editor.canUndo).toBe(false);
    expect(editor.canRedo).toBe(false);
  });

  it('steps back and forward through committed edits', async () => {
    editor.edits.basic.exposure_ev = 1;
    await editor.onCommit('Exposure');
    editor.edits.basic.exposure_ev = 2;
    await editor.onCommit('Exposure');

    expect(editor.canUndo).toBe(true);
    expect(editor.canRedo).toBe(false);

    editor.undo();
    expect(editor.edits.basic.exposure_ev).toBe(1);
    expect(editor.canRedo).toBe(true);

    editor.undo();
    expect(editor.edits.basic.exposure_ev).toBe(0);
    expect(editor.canUndo).toBe(false);

    editor.redo();
    expect(editor.edits.basic.exposure_ev).toBe(1);
  });

  it('saves a replayed state without labelling it as a new action', async () => {
    editor.edits.basic.exposure_ev = 1;
    await editor.onCommit('Exposure');
    expect(mocks.putEdits.mock.calls[0]?.[3]).toBe('Exposure');

    editor.edits.basic.exposure_ev = 2;
    await editor.onCommit('Exposure');
    editor.undo();
    await vi.waitFor(() => expect(mocks.putEdits).toHaveBeenCalledTimes(3));

    expect(mocks.putEdits.mock.calls[2]?.[3]).toBeUndefined();
    expect(editor.canRedo).toBe(true);
    expect(editor.canUndo).toBe(true);
  });

  it('does not add an undo step for a commit that changes nothing', async () => {
    editor.edits.basic.exposure_ev = 1;
    await editor.onCommit('Exposure');
    editor.edits = { ...editor.edits, basic: { ...editor.edits.basic } };
    await editor.onCommit('Auto');

    editor.undo();
    expect(editor.edits.basic.exposure_ev).toBe(0);
    expect(editor.canUndo).toBe(false);
  });

  it('undoes and redoes a jump with server restores instead of saves', async () => {
    mocks.restoreEdits.mockResolvedValueOnce(record('hash-3', -1));
    await jump(3, -1, 5);
    expect(editor.edits.basic.exposure_ev).toBe(-1);
    expect(editor.canUndo).toBe(true);

    mocks.restoreEdits.mockResolvedValueOnce(record('hash-5', 2));
    editor.undo();
    await vi.waitFor(() => expect(editor.edits.basic.exposure_ev).toBe(2));
    expect(mocks.restoreEdits).toHaveBeenLastCalledWith('asset-1', 5);
    expect(editor.canRedo).toBe(true);

    mocks.restoreEdits.mockResolvedValueOnce(record('hash-3', -1));
    editor.redo();
    expect(editor.edits.basic.exposure_ev).toBe(-1);
    await vi.waitFor(() => expect(editor.savedHash).toBe('hash-3'));
    expect(mocks.restoreEdits).toHaveBeenLastCalledWith('asset-1', 3);
    expect(mocks.putEdits).not.toHaveBeenCalled();
  });

  it('replays jumps as saves once a new edit is made', async () => {
    mocks.restoreEdits.mockResolvedValueOnce(record('hash-3', -1));
    await jump(3, -1, 5);
    editor.edits.basic.exposure_ev = 1;
    await editor.onCommit('Exposure');

    editor.undo();
    await vi.waitFor(() => expect(mocks.putEdits).toHaveBeenCalledTimes(2));
    expect(editor.edits.basic.exposure_ev).toBe(-1);

    mocks.deleteEdits.mockClear();
    editor.undo();
    await vi.waitFor(() => expect(mocks.deleteEdits).toHaveBeenCalledTimes(1));
    expect(editor.edits.basic.exposure_ev).toBe(0);
    expect(mocks.restoreEdits).toHaveBeenCalledTimes(1);
  });

  it('keeps a jump step when restores are not tied to a current entry', async () => {
    mocks.restoreEdits.mockResolvedValueOnce(record('hash-3', -1));
    await jump(3, -1);

    mocks.deleteEdits.mockClear();
    editor.undo();
    await vi.waitFor(() => expect(mocks.deleteEdits).toHaveBeenCalledTimes(1));
    expect(editor.edits.basic.exposure_ev).toBe(0);
    expect(mocks.restoreEdits).toHaveBeenCalledTimes(1);
  });

  it('stays on the jump when the undo restore fails', async () => {
    mocks.restoreEdits.mockResolvedValueOnce(record('hash-3', -1));
    await jump(3, -1, 5);

    mocks.restoreEdits.mockRejectedValueOnce(new Error('gone'));
    editor.undo();
    await vi.waitFor(() =>
      expect(toasts.items.at(-1)?.message).toBe('Could not restore the history step: gone')
    );
    expect(editor.canUndo).toBe(true);
    expect(editor.canRedo).toBe(false);
    expect(editor.edits.basic.exposure_ev).toBe(-1);
  });

  it('shows each step at once and keeps the last one when undoing during a jump', async () => {
    mocks.restoreEdits.mockResolvedValueOnce(record('hash-3', -1));
    editor.edits.basic.exposure_ev = 1;
    await editor.onCommit('Exposure');
    await jump(3, -1, 5);
    mocks.putEdits.mockClear();
    mocks.deleteEdits.mockClear();

    const restore = deferred<EditRecord>();
    mocks.restoreEdits.mockReturnValueOnce(restore.promise);
    editor.undo();
    expect(editor.edits.basic.exposure_ev).toBe(1);
    editor.undo();
    expect(editor.edits.basic.exposure_ev).toBe(0);
    await vi.waitFor(() => expect(mocks.restoreEdits).toHaveBeenLastCalledWith('asset-1', 5));
    restore.resolve(record('hash-5', 1));

    await vi.waitFor(() => expect(mocks.deleteEdits).toHaveBeenCalled());
    expect(mocks.deleteEdits.mock.calls.at(-1)).toEqual(['asset-1', undefined, 'hash-5']);
    expect(mocks.putEdits).not.toHaveBeenCalled();
    expect(editor.edits.basic.exposure_ev).toBe(0);
    expect(editor.canUndo).toBe(false);
  });

  it('returns to the step it left when a jump is skipped after a failed save', async () => {
    mocks.restoreEdits.mockResolvedValueOnce(record('hash-3', -1));
    await jump(3, -1, 5);

    const save = deferred<EditRecord>();
    mocks.putEdits.mockReturnValueOnce(save.promise);
    void editor.retrySave();
    editor.undo();
    expect(editor.edits.basic.exposure_ev).toBe(0);

    save.reject(new Error('offline'));
    await vi.waitFor(() => expect(editor.edits.basic.exposure_ev).toBe(-1));
    expect(mocks.restoreEdits).toHaveBeenCalledTimes(1);
    expect(editor.canUndo).toBe(true);
    expect(editor.canRedo).toBe(false);
  });

  it('rolls a row jump back when the restore is skipped', async () => {
    const save = deferred<EditRecord>();
    mocks.putEdits.mockReturnValueOnce(save.promise);
    editor.edits.basic.exposure_ev = 1;
    void editor.onCommit('Exposure');
    const jumping = jump(3, -1, 5);
    expect(editor.edits.basic.exposure_ev).toBe(-1);

    save.reject(new Error('offline'));
    await expect(jumping).resolves.toBe(false);
    expect(editor.edits.basic.exposure_ev).toBe(1);
    expect(mocks.restoreEdits).not.toHaveBeenCalled();
    expect(toasts.items.some((toast) => toast.message.startsWith('Restored'))).toBe(false);
    editor.undo();
    expect(editor.edits.basic.exposure_ev).toBe(0);
  });

  it('ignores the toast undo once a later edit has been made', async () => {
    mocks.restoreEdits.mockResolvedValueOnce(record('hash-3', -1));
    await jump(3, -1, 5);
    const toast = toasts.items.at(-1);
    expect(toast?.message).toBe('Restored Step 3');

    editor.edits.basic.exposure_ev = 2;
    await editor.onCommit('Exposure');
    toast?.action?.run();
    expect(editor.edits.basic.exposure_ev).toBe(2);
    expect(mocks.restoreEdits).toHaveBeenCalledTimes(1);
  });

  it('refreshes the history list after every restore, even to the same saved state', async () => {
    mocks.listEditHistory.mockClear();
    await editor.historySteps.refresh();
    expect(mocks.listEditHistory).toHaveBeenCalledTimes(1);

    mocks.restoreEdits.mockResolvedValueOnce(record('hash-0', 0));
    await jump(3, 0, 5);
    expect(editor.savedHash).toBe('hash-0');
    await Promise.all([editor.historySteps.refresh(), editor.historySteps.refresh()]);
    expect(mocks.listEditHistory).toHaveBeenCalledTimes(2);

    mocks.restoreEdits.mockResolvedValueOnce(record('hash-0', 0));
    editor.undo();
    await vi.waitFor(() => expect(mocks.restoreEdits).toHaveBeenLastCalledWith('asset-1', 5));
    await editor.settled();
    await editor.historySteps.refresh();
    expect(mocks.listEditHistory).toHaveBeenCalledTimes(3);
    await editor.historySteps.refresh();
    expect(mocks.listEditHistory).toHaveBeenCalledTimes(3);
  });

  it('drops the stack when the asset unloads', async () => {
    editor.edits.basic.exposure_ev = 1;
    await editor.onCommit('Exposure');
    expect(editor.canUndo).toBe(true);

    editor.unload();
    expect(editor.canUndo).toBe(false);
    expect(editor.canRedo).toBe(false);
  });
});

describe('history step replacement', () => {
  it('replaces a step only while it is the top one', () => {
    const history = new EditHistory(
      { edits: editsAt(0), onCommit: () => Promise.resolve(), onLive: () => undefined },
      { restoreEntry: () => Promise.resolve(true), restoreSnapshot: () => Promise.resolve(true) }
    );
    const first = history.push(editsAt(0));
    const second = history.push(editsAt(1));
    expect(history.topStep).toBe(second);

    expect(history.replaceStep(first!, editsAt(5))).toBe(false);
    expect(history.replaceStep(second!, editsAt(2))).toBe(true);
    expect(history.push(editsAt(2))).toBeNull();
    expect(history.lastChange).toEqual({ before: editsAt(0), after: editsAt(2) });
  });
});
