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

import { editor } from './editor.svelte';

const JPEG: ExportOptions = {
  format: 'jpeg',
  quality: 90,
  includeExif: true,
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
    editor.lastUpload = { kind: 'success', message: 'Uploaded A_edit.jpg to Immich' };
    editor.lastDownload = { kind: 'error', message: 'Export failed: boom' };
    editor.lastWarnings = ['EXIF dropped'];
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
    expect(editor.lastWarnings).toEqual([]);
    expect(editor.lastImmichOpts).toBeNull();
    expect(editor.lastDownloadOpts).toBeNull();
  });

  it('reports a download next to the button and retries the same request', async () => {
    editor.assetId = 'asset-1';
    mocks.downloadExport.mockRejectedValueOnce(new Error('boom'));
    await editor.onExport(JPEG);
    expect(editor.lastDownload).toEqual({ kind: 'error', message: 'Export failed: boom' });
    expect(editor.error).toBeNull();

    const blob = new Blob();
    mocks.downloadExport.mockResolvedValueOnce({ blob, filename: 'IMG_0001_warm.jpg' });
    await editor.retryExport();
    expect(mocks.downloadExport).toHaveBeenLastCalledWith('asset-1', expect.anything(), JPEG);
    expect(mocks.downloadBlob).toHaveBeenLastCalledWith(blob, 'IMG_0001_warm.jpg');
    expect(editor.lastDownload).toEqual({ kind: 'success', message: 'Saved IMG_0001_warm.jpg' });
  });

  it('falls back to the asset id when the server sends no filename', async () => {
    editor.assetId = 'asset-1';
    mocks.downloadExport.mockResolvedValueOnce({ blob: new Blob(), filename: null });
    await editor.onExport(JPEG);
    expect(mocks.downloadBlob).toHaveBeenLastCalledWith(expect.any(Blob), 'asset-1.jpg');
  });
});
