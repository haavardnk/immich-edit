import { beforeEach, describe, expect, it, vi } from 'vitest';
import { editsToManifest } from '$lib/edits/manifest';
import type { AssetDetail } from '$lib/types/asset';
import { neutralEdits, type EditRecord } from '$lib/types/edits';

const mocks = vi.hoisted(() => ({
  getAsset: vi.fn(),
  getEdits: vi.fn(),
  putEdits: vi.fn(),
  deleteEdits: vi.fn(() => Promise.resolve()),
  restoreEdits: vi.fn(),
  getLensProfile: vi.fn(() => Promise.resolve(null)),
  livePreview: vi.fn(() => new Promise<never>(() => undefined))
}));

vi.mock('$lib/api/assets', () => ({ getAsset: mocks.getAsset }));
vi.mock('$lib/api/edits', () => ({
  getEdits: mocks.getEdits,
  putEdits: mocks.putEdits,
  deleteEdits: mocks.deleteEdits,
  restoreEdits: mocks.restoreEdits,
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

function deferred<T>(): {
  promise: Promise<T>;
  resolve: (value: T) => void;
} {
  let resolve = (_value: T): void => undefined;
  const promise = new Promise<T>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}

describe('editor save coordination', () => {
  beforeEach(async () => {
    editor.unload();
    mocks.putEdits.mockReset();
    mocks.restoreEdits.mockReset();
    mocks.getAsset.mockResolvedValue(asset());
    mocks.getEdits.mockResolvedValue(record('hash-0', 0));
    await editor.load('asset-1');
  });

  it('serializes snapshots and keeps newer local edits authoritative', async () => {
    const first = deferred<EditRecord>();
    const second = deferred<EditRecord>();
    mocks.putEdits.mockImplementationOnce(() => first.promise);
    mocks.putEdits.mockImplementationOnce(() => second.promise);

    editor.edits.basic.exposure_ev = 1;
    const firstSave = editor.onCommit('Exposure');
    editor.edits.basic.exposure_ev = 2;
    const secondSave = editor.onCommit('Exposure');

    await vi.waitFor(() => expect(mocks.putEdits).toHaveBeenCalledTimes(1));
    expect(mocks.putEdits.mock.calls[0][2]).toBe('hash-0');

    first.resolve(record('hash-1', 1));
    await vi.waitFor(() => expect(mocks.putEdits).toHaveBeenCalledTimes(2));
    expect(mocks.putEdits.mock.calls[1][2]).toBe('hash-1');
    expect(editor.edits.basic.exposure_ev).toBe(2);

    second.resolve(record('hash-2', 2));
    await Promise.all([firstSave, secondSave]);

    expect(editor.edits.basic.exposure_ev).toBe(2);
    expect(editor.savedHash).toBe('hash-2');
    expect(editor.saving).toBe(false);
  });

  it('uses a restored history hash for the next save', async () => {
    mocks.restoreEdits.mockResolvedValue(record('hash-restored', -1));
    mocks.putEdits.mockResolvedValue(record('hash-next', 2));

    await editor.restoreHistoryEntry(1);
    expect(editor.edits.basic.exposure_ev).toBe(-1);
    expect(editor.savedHash).toBe('hash-restored');

    editor.edits.basic.exposure_ev = 2;
    await editor.onCommit('Exposure');

    expect(mocks.putEdits).toHaveBeenCalledTimes(1);
    expect(mocks.putEdits.mock.calls[0][2]).toBe('hash-restored');
    expect(editor.savedHash).toBe('hash-next');
  });
});
