import { describe, expect, it, vi } from 'vitest';
import type { ExportOptions } from '#lib/api/export.ts';

const mocks = vi.hoisted(() => ({
  downloadExport: vi.fn(),
  uploadToImmich: vi.fn(),
  downloadBlob: vi.fn(),
  removeTrashed: vi.fn()
}));

vi.mock('#lib/api/export.ts', async (original) => ({
  ...(await original<typeof import('#lib/api/export.ts')>()),
  downloadExport: mocks.downloadExport,
  uploadToImmich: mocks.uploadToImmich
}));
vi.mock('#lib/stores/trash.svelte.ts', () => ({ trash: { removeTrashed: mocks.removeTrashed } }));
vi.mock('#lib/utils/download.ts', () => ({ downloadBlob: mocks.downloadBlob }));

import { editor } from '#lib/stores/editor.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';

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

function rendered(name: string, warnings: string[] = []) {
  return { blob: new Blob(['jpeg']), filename: name, warnings };
}

describe('editor export results', () => {
  it('are cleared when the photo unloads', async () => {
    editor.assetId = 'asset-1';
    mocks.downloadExport.mockResolvedValueOnce(rendered('IMG_0001_warm.jpg'));
    await editor.exports.render(JPEG, false);
    editor.exports.lastUpload = {
      kind: 'success',
      message: 'Uploaded A_edit.jpg to Immich',
      warnings: ['EXIF dropped']
    };
    expect(editor.exports.file).not.toBeNull();

    editor.unload();

    expect(editor.exports.file).toBeNull();
    expect(editor.exports.lastUpload).toBeNull();
    expect(editor.exports.lastDownload).toBeNull();
    await editor.exports.retryRender();
    expect(mocks.downloadExport).toHaveBeenCalledTimes(1);
  });

  it('keeps the rendered file until Save, then marks it saved', async () => {
    editor.assetId = 'asset-1';
    const warnings = ['Metadata not copied: no readable EXIF in the original'];
    mocks.downloadExport.mockResolvedValueOnce(rendered('IMG_0001_warm.jpg', warnings));
    await editor.exports.render(JPEG, false);

    const file = editor.exports.file;
    expect(file).toMatchObject({
      name: 'IMG_0001_warm.jpg',
      saved: false,
      warnings,
      options: JPEG
    });
    expect(mocks.downloadBlob).not.toHaveBeenCalled();

    editor.exports.save();
    expect(mocks.downloadBlob).toHaveBeenLastCalledWith(file?.blob, 'IMG_0001_warm.jpg');
    expect(editor.exports.file?.saved).toBe(true);
    editor.unload();
  });

  it('saves at once when saving automatically', async () => {
    editor.assetId = 'asset-1';
    mocks.downloadBlob.mockClear();
    mocks.downloadExport.mockResolvedValueOnce(rendered('IMG_0001_warm.jpg'));
    await editor.exports.render(JPEG, true);
    expect(mocks.downloadBlob).toHaveBeenCalledTimes(1);
    expect(editor.exports.file?.saved).toBe(true);
    editor.unload();
  });

  it('hides the file once the edit changes', async () => {
    editor.assetId = 'asset-1';
    mocks.downloadExport.mockResolvedValueOnce(rendered('IMG_0001_warm.jpg'));
    await editor.exports.render(JPEG, false);
    expect(editor.exports.file).not.toBeNull();

    editor.edits = { ...editor.edits, basic: { ...editor.edits.basic, exposure_ev: 0.5 } };
    expect(editor.exports.file).toBeNull();
    editor.unload();
  });

  it('reports a failed render next to the button and retries the same request', async () => {
    editor.assetId = 'asset-1';
    mocks.downloadExport.mockRejectedValueOnce(new Error('boom'));
    await editor.exports.render(JPEG, false);
    expect(editor.exports.lastDownload).toEqual({
      kind: 'error',
      message: 'Export failed: boom',
      warnings: []
    });
    expect(editor.error).toBeNull();

    mocks.downloadExport.mockResolvedValueOnce(rendered('IMG_0001_warm.jpg'));
    await editor.exports.retryRender();
    expect(mocks.downloadExport).toHaveBeenLastCalledWith('asset-1', expect.anything(), JPEG);
    expect(editor.exports.lastDownload).toBeNull();
    expect(editor.exports.file?.name).toBe('IMG_0001_warm.jpg');
    editor.unload();
  });

  it('falls back to the asset id when the server sends no filename', async () => {
    editor.assetId = 'asset-1';
    mocks.downloadExport.mockResolvedValueOnce({ blob: new Blob(), filename: null, warnings: [] });
    await editor.exports.render(JPEG, true);
    expect(mocks.downloadBlob).toHaveBeenLastCalledWith(expect.any(Blob), 'asset-1.jpg');
    editor.unload();
  });

  it('drops a render that finishes after the photo changed', async () => {
    editor.assetId = 'asset-1';
    let finish: (value: unknown) => void = () => {};
    mocks.downloadExport.mockReturnValueOnce(new Promise((resolve) => (finish = resolve)));
    const render = editor.exports.render(JPEG, true);
    editor.unload();
    editor.assetId = 'asset-2';
    finish(rendered('IMG_0001_warm.jpg'));
    await render;
    expect(editor.exports.file).toBeNull();
    editor.unload();
  });

  it('keeps the next photo busy while the last photo render finishes', async () => {
    editor.assetId = 'asset-1';
    let finishA: (value: unknown) => void = () => {};
    mocks.downloadExport.mockReturnValueOnce(new Promise((resolve) => (finishA = resolve)));
    const renderA = editor.exports.render(JPEG, false);
    editor.unload();
    editor.assetId = 'asset-2';
    let finishB: (value: unknown) => void = () => {};
    mocks.downloadExport.mockReturnValueOnce(new Promise((resolve) => (finishB = resolve)));
    const renderB = editor.exports.render(JPEG, false);

    finishA(rendered('A_warm.jpg'));
    await renderA;
    expect(editor.exports.rendering).toBe(true);
    expect(editor.exports.file).toBeNull();

    finishB(rendered('B_warm.jpg'));
    await renderB;
    expect(editor.exports.rendering).toBe(false);
    expect(editor.exports.file?.name).toBe('B_warm.jpg');
    editor.unload();
  });

  it('leaves the next photo alone when an earlier upload finishes', async () => {
    editor.assetId = 'asset-1';
    let finish: (value: unknown) => void = () => {};
    mocks.uploadToImmich.mockReturnValueOnce(new Promise((resolve) => (finish = resolve)));
    const upload = editor.exports.upload({
      ...JPEG,
      albumIds: [],
      tagIds: [],
      favorite: true,
      stackWithOriginal: false,
      stackPrimary: 'edited',
      replaceOriginal: false,
      onConflict: 'keep_both'
    });
    editor.unload();
    editor.assetId = 'asset-2';
    finish({
      asset_id: 'new',
      filename: 'IMG_0001_edit.jpg',
      status: 'created',
      warnings: [],
      original_trashed: false
    });
    await expect(upload).resolves.toBe(false);
    expect(editor.exports.lastUpload).toBeNull();
    expect(editor.asset).toBeNull();
    editor.unload();
  });

  it('takes a replaced original out of the grid with an undo', async () => {
    editor.assetId = '11111111-1111-1111-1111-111111111111_2';
    mocks.uploadToImmich.mockResolvedValueOnce({
      asset_id: 'new',
      filename: 'IMG_0001_edit.jpg',
      status: 'created',
      warnings: [],
      original_trashed: true
    });
    const trashed = await editor.exports.upload({
      ...JPEG,
      albumIds: [],
      tagIds: [],
      favorite: false,
      stackWithOriginal: false,
      stackPrimary: 'edited',
      replaceOriginal: true,
      onConflict: 'keep_both'
    });
    const message = 'Uploaded IMG_0001_edit.jpg to Immich; the original moved to the Immich trash';
    expect(mocks.removeTrashed).toHaveBeenCalledWith(
      ['11111111-1111-1111-1111-111111111111'],
      message
    );
    expect(editor.exports.lastUpload).toEqual({ kind: 'success', message, warnings: [] });
    expect(trashed).toBe(true);
  });

  it('warns when an upload was skipped and lists kept previous exports', async () => {
    editor.assetId = 'asset-1';
    const upload = {
      ...JPEG,
      albumIds: [],
      tagIds: [],
      favorite: false,
      stackWithOriginal: false,
      stackPrimary: 'edited' as const,
      replaceOriginal: false,
      onConflict: 'skip' as const
    };
    mocks.uploadToImmich.mockResolvedValueOnce({
      asset_id: 'old',
      filename: 'IMG_0001_edit.jpg',
      status: 'skipped',
      warnings: [],
      original_trashed: false
    });
    await editor.exports.upload(upload);
    expect(editor.exports.lastUpload).toEqual({
      kind: 'warning',
      message: 'Not uploaded: this photo is already in Immich from an earlier export',
      warnings: []
    });

    const kept = 'Previous export kept: it has its own edits';
    mocks.uploadToImmich.mockResolvedValueOnce({
      asset_id: 'new',
      filename: 'IMG_0001_edit.jpg',
      status: 'created',
      warnings: [kept],
      original_trashed: false
    });
    await editor.exports.upload({ ...upload, onConflict: 'replace' });
    expect(editor.exports.lastUpload).toEqual({
      kind: 'success',
      message: 'Uploaded IMG_0001_edit.jpg to Immich',
      warnings: [kept]
    });
    expect(toasts.items.at(-1)).toMatchObject({ kind: 'warn', description: kept });
    editor.unload();
  });

  it('takes a replaced earlier export out of the grid with an undo', async () => {
    editor.assetId = 'asset-1';
    mocks.removeTrashed.mockClear();
    mocks.uploadToImmich.mockResolvedValueOnce({
      asset_id: 'new',
      filename: 'IMG_0001_edit.jpg',
      status: 'created',
      warnings: [],
      original_trashed: false,
      replaced_asset_id: 'old'
    });
    await editor.exports.upload({
      ...JPEG,
      albumIds: [],
      tagIds: [],
      favorite: false,
      stackWithOriginal: false,
      stackPrimary: 'edited',
      replaceOriginal: false,
      onConflict: 'replace'
    });
    expect(mocks.removeTrashed).toHaveBeenCalledWith(
      ['old'],
      'Uploaded IMG_0001_edit.jpg to Immich; the earlier export moved to the Immich trash'
    );
    editor.unload();
  });
});
