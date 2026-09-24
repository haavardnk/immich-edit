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
  colorSpace: 'srgb'
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
      stackPrimary: 'edited',
      filenameSuffix: '_edit'
    };
    editor.lastDownloadRequest = { opts: JPEG, suffix: '_edit' };

    editor.unload();

    expect(editor.lastUpload).toBeNull();
    expect(editor.lastDownload).toBeNull();
    expect(editor.lastWarnings).toEqual([]);
    expect(editor.lastImmichOpts).toBeNull();
    expect(editor.lastDownloadRequest).toBeNull();
  });

  it('reports a download next to the button and retries the same request', async () => {
    editor.assetId = 'asset-1';
    mocks.downloadExport.mockRejectedValueOnce(new Error('boom'));
    await editor.onExport({ opts: JPEG, suffix: '_warm' });
    expect(editor.lastDownload).toEqual({ kind: 'error', message: 'Export failed: boom' });
    expect(editor.error).toBeNull();

    mocks.downloadExport.mockResolvedValueOnce(new Blob());
    await editor.retryExport();
    expect(mocks.downloadBlob).toHaveBeenLastCalledWith(expect.any(Blob), 'asset-1_warm.jpg');
    expect(editor.lastDownload).toEqual({ kind: 'success', message: 'Saved asset-1_warm.jpg' });
  });
});
