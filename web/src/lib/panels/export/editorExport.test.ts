import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  backToGrid: vi.fn(),
  submitPresetExport: vi.fn(),
  showJob: vi.fn()
}));

vi.mock('$app/state', () => ({
  page: { url: new URL('http://localhost/edit/a1?from=%2Falbums%2Fx') }
}));
vi.mock('#lib/browse/backToGrid.ts', () => ({ backToGrid: mocks.backToGrid }));
vi.mock('#lib/panels/export/presetExport.ts', () => ({
  submitPresetExport: mocks.submitPresetExport
}));
vi.mock('#lib/stores/jobs.svelte.ts', () => ({ jobs: { showJob: mocks.showJob } }));

import { EditorExporter } from '#lib/panels/export/editorExport.svelte.ts';
import { exportPresets } from '#lib/panels/export/exportPresets.svelte.ts';
import { exportSettings } from '#lib/panels/export/exportSettings.svelte.ts';
import { uploadOptions } from '#lib/panels/export/scope.ts';
import { baseOptions, defaultExportForm } from '#lib/panels/export/settings.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import { immichCapabilities } from '#lib/stores/immichCapabilities.svelte.ts';
import { mdiCloudUpload, mdiExport } from '@mdi/js';

beforeEach(() => {
  editor.assetId = 'a1';
  mocks.backToGrid.mockResolvedValue(undefined);
});

afterEach(() => {
  editor.assetId = null;
  editor.exports.reset();
  exportSettings.form = defaultExportForm();
  exportSettings.destination = 'download';
  exportSettings.saveAutomatically = false;
  immichCapabilities.trash = false;
  vi.restoreAllMocks();
  mocks.backToGrid.mockReset();
  mocks.submitPresetExport.mockReset();
  mocks.showJob.mockReset();
  exportPresets.presets = [];
  editor.saveError = null;
});

describe('EditorExporter download', () => {
  it('labels the button by format and busy state', () => {
    const exporter = new EditorExporter();
    expect(exporter.label).toBe('Export JPEG');
    expect(exporter.icon).toBe(mdiExport);
    expect(exporter.disabled).toBe(false);

    editor.exports.rendering = true;
    expect(exporter.busy).toBe(true);
    expect(exporter.label).toBe('Exporting…');
    expect(exporter.disabled).toBe(true);
  });

  it('is disabled without a photo', () => {
    editor.assetId = null;
    expect(new EditorExporter().disabled).toBe(true);
  });

  it('starts and retries a download, saving only when the device says so', () => {
    const render = vi.spyOn(editor.exports, 'render').mockResolvedValue();
    const retryRender = vi.spyOn(editor.exports, 'retryRender').mockResolvedValue();
    const exporter = new EditorExporter();

    exporter.start();
    expect(render).toHaveBeenLastCalledWith(baseOptions(exportSettings.form), false);
    exportSettings.saveAutomatically = true;
    exporter.start();
    expect(render).toHaveBeenLastCalledWith(baseOptions(exportSettings.form), true);

    exporter.retry();
    expect(retryRender).toHaveBeenCalledTimes(1);
    expect(mocks.backToGrid).not.toHaveBeenCalled();
  });

  it('shows the last failed download', () => {
    const result = { kind: 'error' as const, message: 'Export failed: boom', warnings: [] };
    editor.exports.lastDownload = result;
    expect(new EditorExporter().result).toEqual(result);
  });

  it('drops the rendered file once the export settings change', () => {
    vi.spyOn(editor.exports, 'file', 'get').mockReturnValue({
      blob: new Blob(),
      name: 'IMG_0001_edit.jpg',
      options: baseOptions(exportSettings.form),
      edits: editor.edits,
      size: null,
      renderedAt: new Date(),
      renderMs: 10,
      saved: false,
      warnings: []
    });
    const exporter = new EditorExporter();
    expect(exporter.file?.name).toBe('IMG_0001_edit.jpg');
    const quality = exportSettings.form.quality;
    exportSettings.form.quality = 50;
    expect(exporter.file).toBeNull();
    exportSettings.form.quality = quality;
  });

  it('folds the form while a render runs', () => {
    const exporter = new EditorExporter();
    expect(exporter.folded).toBe(false);
    editor.exports.rendering = true;
    expect(exporter.folded).toBe(true);
    exportSettings.destination = 'immich';
    expect(exporter.folded).toBe(false);
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

    editor.exports.uploading = true;
    expect(exporter.label).toBe('Uploading…');
  });

  it('uploads, resets replace original and returns to the grid', async () => {
    const upload = vi.spyOn(editor.exports, 'upload').mockResolvedValue(true);
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
    const upload = vi.spyOn(editor.exports, 'upload').mockResolvedValue(false);
    new EditorExporter().start();
    await upload.mock.results[0]?.value;
    await Promise.resolve();
    expect(mocks.backToGrid).not.toHaveBeenCalled();
  });

  it('uploads a rendered file with its options and the Immich choices', () => {
    exportSettings.destination = 'download';
    const options = baseOptions(exportSettings.form);
    vi.spyOn(editor.exports, 'file', 'get').mockReturnValue({
      blob: new Blob(),
      name: 'IMG_0001_edit.jpg',
      options,
      edits: editor.edits,
      size: null,
      renderedAt: new Date(),
      renderMs: 2400,
      saved: false,
      warnings: []
    });
    const upload = vi.spyOn(editor.exports, 'upload').mockResolvedValue(false);
    const exporter = new EditorExporter();
    exporter.replaceOriginal = true;

    exporter.uploadFile();
    expect(upload).toHaveBeenCalledWith({
      ...uploadOptions(exportSettings.form, exporter.scope, false),
      ...options
    });
  });

  it('returns to the grid after a successful retry', async () => {
    const retry = vi.spyOn(editor.exports, 'retryUpload').mockResolvedValue(true);
    new EditorExporter().retry();
    expect(retry).toHaveBeenCalledTimes(1);
    await vi.waitFor(() => expect(mocks.backToGrid).toHaveBeenCalledWith('a1', '/albums/x'));
  });
});

describe('EditorExporter presets', () => {
  const web = {
    id: 'web',
    name: 'Web',
    form: {},
    destination: 'download' as const,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z'
  };
  const print = { ...web, id: 'print', name: 'Print', form: { quality: 70 } };

  it('exports one ticked preset directly with its settings and saves it', async () => {
    exportPresets.presets = [web, print];
    const render = vi.spyOn(editor.exports, 'render').mockResolvedValue();
    const exporter = new EditorExporter();
    exporter.presets.toggle('print', true);

    await expect(exporter.exportPresets()).resolves.toBe(true);
    expect(exportSettings.form.quality).toBe(70);
    expect(render).toHaveBeenCalledWith(baseOptions(exportSettings.form), true);
    expect(mocks.submitPresetExport).not.toHaveBeenCalled();
    expect(exporter.presets.picked).toEqual([]);
  });

  it('keeps one ticked preset while an export runs', async () => {
    exportPresets.presets = [web];
    editor.exports.rendering = true;
    const exporter = new EditorExporter();
    exporter.presets.toggle('web', true);

    await expect(exporter.exportPresets()).resolves.toBe(false);
    expect(exporter.presets.picked).toEqual([web]);
  });

  it('exports several ticked presets of the saved photo as a job', async () => {
    exportPresets.presets = [web, print];
    mocks.submitPresetExport.mockResolvedValue([{ destination: 'download', job: { id: 'job-1' } }]);
    const exporter = new EditorExporter();
    exporter.presets.toggle('web', true);
    exporter.presets.toggle('print', true);

    await expect(exporter.exportPresets()).resolves.toBe(true);
    expect(mocks.submitPresetExport).toHaveBeenCalledWith(['a1'], [web, print], exporter.scope);
    expect(mocks.showJob).toHaveBeenCalledWith('job-1');
    expect(exporter.presets.picked).toEqual([]);
    expect(exporter.exportingPresets).toBe(false);
  });

  it('refuses several presets while the edits are not saved', async () => {
    exportPresets.presets = [web, print];
    editor.saveError = 'offline';
    const exporter = new EditorExporter();
    exporter.presets.toggle('web', true);
    exporter.presets.toggle('print', true);

    await expect(exporter.exportPresets()).resolves.toBe(false);
    expect(mocks.submitPresetExport).not.toHaveBeenCalled();
    expect(exporter.presets.picked).toEqual([web, print]);
  });
});
