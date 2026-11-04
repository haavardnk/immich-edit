import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({ backToGrid: vi.fn() }));

vi.mock('$app/state', () => ({
  page: { url: new URL('http://localhost/edit/a1?from=%2Falbums%2Fx') }
}));
vi.mock('#lib/browse/backToGrid.ts', () => ({ backToGrid: mocks.backToGrid }));

import { EditorExporter } from '#lib/panels/export/editorExport.svelte.ts';
import { exportSettings } from '#lib/panels/export/exportSettings.svelte.ts';
import { uploadOptions } from '#lib/panels/export/scope.ts';
import { baseOptions } from '#lib/panels/export/settings.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import { immichCapabilities } from '#lib/stores/immichCapabilities.svelte.ts';
import { mdiCloudUpload, mdiExport } from '@mdi/js';

beforeEach(() => {
  editor.assetId = 'a1';
  mocks.backToGrid.mockResolvedValue(undefined);
});

afterEach(() => {
  editor.assetId = null;
  editor.exporting = false;
  editor.exportingToImmich = false;
  editor.lastDownload = null;
  editor.lastUpload = null;
  exportSettings.destination = 'download';
  immichCapabilities.trash = false;
  vi.restoreAllMocks();
  mocks.backToGrid.mockReset();
});

describe('EditorExporter download', () => {
  it('labels the button by format and busy state', () => {
    const exporter = new EditorExporter();
    expect(exporter.label).toBe('Export JPEG');
    expect(exporter.icon).toBe(mdiExport);
    expect(exporter.disabled).toBe(false);

    editor.exporting = true;
    expect(exporter.busy).toBe(true);
    expect(exporter.label).toBe('Exporting…');
    expect(exporter.disabled).toBe(true);
  });

  it('is disabled without a photo', () => {
    editor.assetId = null;
    expect(new EditorExporter().disabled).toBe(true);
  });

  it('starts and retries a download', () => {
    const onExport = vi.spyOn(editor, 'onExport').mockResolvedValue();
    const retryExport = vi.spyOn(editor, 'retryExport').mockResolvedValue();
    const exporter = new EditorExporter();

    exporter.start();
    expect(onExport).toHaveBeenCalledWith(baseOptions(exportSettings.form));

    exporter.retry();
    expect(retryExport).toHaveBeenCalledTimes(1);
    expect(mocks.backToGrid).not.toHaveBeenCalled();
  });

  it('shows the last download', () => {
    const result = { kind: 'success' as const, message: 'Saved', warnings: [] };
    editor.lastDownload = result;
    expect(new EditorExporter().result).toEqual(result);
  });
});

describe('EditorExporter upload', () => {
  beforeEach(() => {
    exportSettings.destination = 'immich';
    immichCapabilities.trash = true;
  });

  it('labels the button for Immich', () => {
    const exporter = new EditorExporter();
    expect(exporter.label).toBe('Upload JPEG to Immich');
    expect(exporter.icon).toBe(mdiCloudUpload);

    editor.exportingToImmich = true;
    expect(exporter.label).toBe('Uploading…');
  });

  it('uploads, resets replace original and returns to the grid', async () => {
    const upload = vi.spyOn(editor, 'onUploadToImmich').mockResolvedValue(true);
    const exporter = new EditorExporter();
    exporter.replaceOriginal = true;
    const expected = uploadOptions(exportSettings.form, exporter.scope, true);

    exporter.start();
    expect(upload).toHaveBeenCalledWith(expected);
    expect(expected.replaceOriginal).toBe(true);
    expect(exporter.replaceOriginal).toBe(false);
    await vi.waitFor(() => expect(mocks.backToGrid).toHaveBeenCalledWith('a1', '/albums/x'));
  });

  it('stays in the editor when the upload fails', async () => {
    const upload = vi.spyOn(editor, 'onUploadToImmich').mockResolvedValue(false);
    new EditorExporter().start();
    await upload.mock.results[0]?.value;
    await Promise.resolve();
    expect(mocks.backToGrid).not.toHaveBeenCalled();
  });

  it('returns to the grid after a successful retry', async () => {
    const retry = vi.spyOn(editor, 'retryUpload').mockResolvedValue(true);
    new EditorExporter().retry();
    expect(retry).toHaveBeenCalledTimes(1);
    await vi.waitFor(() => expect(mocks.backToGrid).toHaveBeenCalledWith('a1', '/albums/x'));
  });
});
