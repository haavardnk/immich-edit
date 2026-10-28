import { beforeEach, describe, expect, it, vi } from 'vitest';
import { ConflictError } from '$lib/api/client';
import { editsToManifest } from '$lib/edits/manifest';
import type { AssetDetail } from '$lib/types/asset';
import { neutralEdits, type EditRecord } from '$lib/types/edits';
import type { PreviewMeta } from '$lib/types/preview';

const mocks = vi.hoisted(() => ({
  getAsset: vi.fn(),
  getEdits: vi.fn(),
  putEdits: vi.fn(),
  deleteEdits: vi.fn(() => Promise.resolve()),
  restoreEdits: vi.fn(),
  getLensProfile: vi.fn(() => Promise.resolve(null)),
  livePreview: vi.fn(() => new Promise<never>(() => undefined)),
  startDenoise: vi.fn(() => Promise.resolve({ state: 'queued' })),
  getDenoiseStatus: vi.fn(() => Promise.resolve({ state: 'queued' })),
  listEditedAssets: vi.fn()
}));

vi.mock('$lib/api/assets', () => ({ getAsset: mocks.getAsset }));
vi.mock('$lib/api/edits', () => ({
  getEdits: mocks.getEdits,
  putEdits: mocks.putEdits,
  deleteEdits: mocks.deleteEdits,
  restoreEdits: mocks.restoreEdits,
  listEditedAssets: mocks.listEditedAssets,
  autoEdits: vi.fn()
}));
vi.mock('$lib/api/lensProfile', () => ({ getLensProfile: mocks.getLensProfile }));
vi.mock('$lib/api/denoise', () => ({
  startDenoise: mocks.startDenoise,
  getDenoiseStatus: mocks.getDenoiseStatus
}));
vi.mock('$lib/api/preview', async (original) => ({
  ...(await original<typeof import('$lib/api/preview')>()),
  livePreview: mocks.livePreview
}));

import { editor } from '$lib/stores/editor.svelte';
import { editedThumbs } from '$lib/stores/editedThumbs.svelte';

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

function record(hash: string, exposure: number, aiDenoise = false): EditRecord {
  const edits = neutralEdits();
  edits.basic.exposure_ev = exposure;
  edits.detail.ai_denoise = aiDenoise;
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

function previewMeta(isRaw: boolean): PreviewMeta {
  return {
    asset_id: 'asset-1',
    width: 4,
    height: 4,
    source_w: 4,
    source_h: 4,
    renderer: 'cpu',
    is_raw: isRaw,
    histogram: { r: [], g: [], b: [], l: [] },
    has_scopes: false
  };
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
    expect(mocks.putEdits.mock.calls[0]?.[2]).toBe('hash-0');

    first.resolve(record('hash-1', 1));
    await vi.waitFor(() => expect(mocks.putEdits).toHaveBeenCalledTimes(2));
    expect(mocks.putEdits.mock.calls[1]?.[2]).toBe('hash-1');
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
    expect(mocks.putEdits.mock.calls[0]?.[2]).toBe('hash-restored');
    expect(editor.savedHash).toBe('hash-next');
  });

  it('starts the AI denoise bake when a restored history entry turns it on', async () => {
    mocks.startDenoise.mockClear();
    mocks.restoreEdits.mockResolvedValue(record('hash-restored', 0, true));
    editor.meta = previewMeta(true);

    await editor.restoreHistoryEntry(1);

    expect(editor.edits.detail.ai_denoise).toBe(true);
    expect(mocks.startDenoise).toHaveBeenCalledWith('asset-1');
    await vi.waitFor(() => expect(editor.aiDenoise.status).toEqual({ state: 'queued' }));
    editor.unload();
  });

  it('refreshes the edited thumbnail revision when a bake lands', async () => {
    vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout'] });
    try {
      mocks.listEditedAssets.mockResolvedValue({
        render_revision: 'base',
        items: [{ id: 'asset-1', hash: 'hash-next', updated_at: '', revision: 'base-dn1f' }]
      });
      mocks.getDenoiseStatus.mockResolvedValueOnce({ state: 'ready' });
      mocks.putEdits.mockResolvedValue(record('hash-next', 0, true));
      editor.meta = previewMeta(true);

      editor.edits.detail.ai_denoise = true;
      editor.edits.detail.ai_denoise_strength = 75;
      await editor.onCommit('AI Denoise');
      await vi.waitFor(() => expect(editor.aiDenoise.status).toEqual({ state: 'queued' }));
      await vi.advanceTimersByTimeAsync(1500);

      await vi.waitFor(() => expect(editedThumbs.getRevision('asset-1')).toBe('base-dn1f'));
    } finally {
      vi.useRealTimers();
      editor.unload();
    }
  });

  it.each([
    [true, 75, true, 1],
    [true, 0, true, 0],
    [false, 75, true, 0],
    [true, 75, false, 0]
  ])(
    'with AI denoise %s at strength %i on raw %s starts %i bakes',
    async (on, strength, raw, starts) => {
      mocks.startDenoise.mockClear();
      mocks.putEdits.mockResolvedValue(record('hash-next', 0));
      editor.meta = previewMeta(raw);

      editor.edits.detail.ai_denoise = on;
      editor.edits.detail.ai_denoise_strength = strength;
      await editor.onCommit('AI Denoise');

      expect(mocks.startDenoise).toHaveBeenCalledTimes(starts);
      editor.unload();
    }
  );

  it('sends the known hash when a reset deletes the sidecar', async () => {
    mocks.deleteEdits.mockClear();

    editor.edits = neutralEdits();
    await editor.onCommit('Reset');

    expect(mocks.deleteEdits).toHaveBeenCalledTimes(1);
    expect(mocks.deleteEdits.mock.calls[0]).toEqual(['asset-1', 'Reset', 'hash-0']);
    expect(editor.savedHash).toBe('');
  });

  it('keeps local edits when a reset conflicts', async () => {
    mocks.deleteEdits.mockClear();
    mocks.deleteEdits.mockRejectedValueOnce(
      new ConflictError('conflict', record('hash-remote', 3))
    );

    editor.edits = neutralEdits();
    await editor.onCommit('Reset');

    expect(editor.savedHash).toBe('hash-remote');
    expect(editor.saveError).toContain('changed elsewhere');
  });
});
