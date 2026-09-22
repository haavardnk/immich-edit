import { beforeEach, describe, expect, it, vi } from 'vitest';
import { editsToManifest } from '$lib/edits/manifest';
import type { AssetDetail } from '$lib/types/asset';
import { neutralEdits, type EditRecord } from '$lib/types/edits';

const mocks = vi.hoisted(() => ({
  getAsset: vi.fn(),
  getEdits: vi.fn(),
  putEdits: vi.fn(),
  sampleWhiteBalance: vi.fn(),
  autoWhiteBalance: vi.fn(),
  getLensProfile: vi.fn(() => Promise.resolve(null)),
  livePreview: vi.fn(() => new Promise<never>(() => undefined))
}));

vi.mock('$lib/api/assets', () => ({ getAsset: mocks.getAsset }));
vi.mock('$lib/api/edits', () => ({
  getEdits: mocks.getEdits,
  putEdits: mocks.putEdits,
  deleteEdits: vi.fn(() => Promise.resolve()),
  restoreEdits: vi.fn(),
  autoEdits: vi.fn(),
  sampleWhiteBalance: mocks.sampleWhiteBalance,
  autoWhiteBalance: mocks.autoWhiteBalance
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

function record(hash: string): EditRecord {
  return {
    schema_version: 1,
    asset_id: 'asset-1',
    renderer_version: 'test',
    manifest: editsToManifest(neutralEdits()),
    hash,
    updated_at: '2026-09-01T00:00:00Z',
    immich_updated_at: null,
    immich_checksum: null
  };
}

describe('white balance picker', () => {
  beforeEach(async () => {
    editor.unload();
    mocks.sampleWhiteBalance.mockReset();
    mocks.autoWhiteBalance.mockReset();
    mocks.putEdits.mockResolvedValue(record('hash-1'));
    mocks.getAsset.mockResolvedValue(asset());
    mocks.getEdits.mockResolvedValue(record('hash-0'));
    await editor.load('asset-1');
  });

  it('applies and commits a sampled neutral point', async () => {
    mocks.sampleWhiteBalance.mockResolvedValue({ wb_temp: -12, wb_tint: 4 });
    editor.toggleWbPicker();
    expect(editor.wbPicking).toBe(true);

    await editor.pickWhiteBalance(0.25, 0.75);

    expect(mocks.sampleWhiteBalance.mock.calls[0]?.slice(0, 3)).toEqual(['asset-1', 0.25, 0.75]);
    expect(editor.edits.basic.wb_temp).toBe(-12);
    expect(editor.edits.basic.wb_tint).toBe(4);
    expect(editor.wbPicking).toBe(false);
    expect(mocks.putEdits).toHaveBeenCalledTimes(1);
  });

  it('applies auto white balance without leaving picker mode on', async () => {
    mocks.autoWhiteBalance.mockResolvedValue({ wb_temp: 8, wb_tint: -3 });

    await editor.onAutoWhiteBalance();

    expect(editor.edits.basic.wb_temp).toBe(8);
    expect(editor.edits.basic.wb_tint).toBe(-3);
    expect(editor.wbPicking).toBe(false);
    expect(editor.wbBusy).toBe(false);
  });

  it('keeps edits unchanged and surfaces an error when no colour is usable', async () => {
    mocks.sampleWhiteBalance.mockRejectedValue(new Error('no usable colour at that point'));
    editor.toggleWbPicker();

    await editor.pickWhiteBalance(0.5, 0.5);

    expect(editor.edits.basic.wb_temp).toBe(0);
    expect(editor.error).toBe('no usable colour at that point');
    expect(editor.wbBusy).toBe(false);
    expect(mocks.putEdits).not.toHaveBeenCalled();
  });
});
