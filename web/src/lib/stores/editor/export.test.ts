import { describe, expect, it, vi } from 'vitest';
import type { ExportOptions } from '$lib/api/export';

const mocks = vi.hoisted(() => ({
  downloadExport: vi.fn(),
  downloadBlob: vi.fn()
}));

vi.mock('$lib/api/export', async (original) => ({
  ...(await original<typeof import('$lib/api/export')>()),
  downloadExport: mocks.downloadExport
}));
vi.mock('$lib/utils/download', () => ({ downloadBlob: mocks.downloadBlob }));

import { editor } from '$lib/stores/editor.svelte';

const JPEG: ExportOptions = {
  format: 'jpeg',
  quality: 90,
  metadata: 'all',
  bitDepth: '8',
  pngCompression: 'default',
  tiffCompression: 'lzw',
  lossless: false,
  colorSpace: 'srgb',
  filenameTemplate: '{name}_warm',
  resize: null,
  sharpen: null,
  watermark: null
};

describe('editor export results', () => {
  it('are cleared when the photo unloads', () => {
    editor.lastUpload = {
      kind: 'success',
      message: 'Uploaded A_edit.jpg to Immich',
      warnings: ['EXIF dropped']
    };
    editor.lastDownload = { kind: 'error', message: 'Export failed: boom', warnings: [] };
    editor.lastImmichOpts = {
      ...JPEG,
      albumIds: [],
      tagIds: [],
      favorite: false,
      stackWithOriginal: false,
      stackPrimary: 'edited'
    };
    editor.lastDownloadOpts = JPEG;

    editor.unload();

    expect(editor.lastUpload).toBeNull();
    expect(editor.lastDownload).toBeNull();
    expect(editor.lastImmichOpts).toBeNull();
    expect(editor.lastDownloadOpts).toBeNull();
  });

  it('reports a download next to the button and retries the same request', async () => {
    editor.assetId = 'asset-1';
    mocks.downloadExport.mockRejectedValueOnce(new Error('boom'));
    await editor.onExport(JPEG);
    expect(editor.lastDownload).toEqual({
      kind: 'error',
      message: 'Export failed: boom',
      warnings: []
    });
    expect(editor.error).toBeNull();

    const blob = new Blob();
    const warnings = ['Metadata not copied: no readable EXIF in the original'];
    mocks.downloadExport.mockResolvedValueOnce({ blob, filename: 'IMG_0001_warm.jpg', warnings });
    await editor.retryExport();
    expect(mocks.downloadExport).toHaveBeenLastCalledWith('asset-1', expect.anything(), JPEG);
    expect(mocks.downloadBlob).toHaveBeenLastCalledWith(blob, 'IMG_0001_warm.jpg');
    expect(editor.lastDownload).toEqual({
      kind: 'success',
      message: 'Saved IMG_0001_warm.jpg',
      warnings
    });
  });

  it('falls back to the asset id when the server sends no filename', async () => {
    editor.assetId = 'asset-1';
    mocks.downloadExport.mockResolvedValueOnce({ blob: new Blob(), filename: null, warnings: [] });
    await editor.onExport(JPEG);
    expect(mocks.downloadBlob).toHaveBeenLastCalledWith(expect.any(Blob), 'asset-1.jpg');
  });
});
