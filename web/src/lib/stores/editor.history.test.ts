import { beforeEach, describe, expect, it, vi } from 'vitest';
import { editsToManifest } from '$lib/edits/manifest';
import type { AssetDetail } from '$lib/types/asset';
import { neutralEdits, type EditRecord } from '$lib/types/edits';

const mocks = vi.hoisted(() => ({
  getAsset: vi.fn(),
  getEdits: vi.fn(),
  putEdits: vi.fn(),
  deleteEdits: vi.fn(() => Promise.resolve()),
  getLensProfile: vi.fn(() => Promise.resolve(null)),
  livePreview: vi.fn(() => new Promise<never>(() => undefined))
}));

vi.mock('$lib/api/assets', () => ({ getAsset: mocks.getAsset }));
vi.mock('$lib/api/edits', () => ({
  getEdits: mocks.getEdits,
  putEdits: mocks.putEdits,
  deleteEdits: mocks.deleteEdits,
  restoreEdits: vi.fn(),
  autoEdits: vi.fn()
}));
vi.mock('$lib/api/lensProfile', () => ({ getLensProfile: mocks.getLensProfile }));
vi.mock('$lib/api/preview', async (original) => ({
  ...(await original<typeof import('$lib/api/preview')>()),
  livePreview: mocks.livePreview
}));

import { editor } from './editor.svelte';

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

function record(hash: string, exposure: number): EditRecord {
  const edits = neutralEdits();
  edits.basic.exposure_ev = exposure;
  return {
    schema_version: 1,
    asset_id: 'asset-1',
    renderer_version: 'test',
    manifest: editsToManifest(edits),
    hash,
    updated_at: '2026-09-01T00:00:00Z',
    immich_updated_at: null,
    immich_checksum: null
  };
}

describe('editor history', () => {
  beforeEach(async () => {
    editor.unload();
    mocks.putEdits.mockReset();
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

  it('drops the stack when the asset unloads', async () => {
    editor.edits.basic.exposure_ev = 1;
    await editor.onCommit('Exposure');
    expect(editor.canUndo).toBe(true);

    editor.unload();
    expect(editor.canUndo).toBe(false);
    expect(editor.canRedo).toBe(false);
  });
});
