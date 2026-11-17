import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  createImmichExportJob: vi.fn(),
  createZipExportJob: vi.fn(),
  runBulkJob: vi.fn(),
  showJob: vi.fn(),
  downloadWhenDone: vi.fn(),
  downloadStoredExport: vi.fn(),
  downloadBlob: vi.fn(),
  submitPresetExport: vi.fn(),
  getEdits: vi.fn(),
  uploadToImmich: vi.fn()
}));

vi.mock('#lib/api/jobs.ts', async (original) => ({
  ...(await original<typeof import('#lib/api/jobs.ts')>()),
  createImmichExportJob: mocks.createImmichExportJob,
  createZipExportJob: mocks.createZipExportJob
}));
vi.mock('#lib/api/bulkJob.ts', () => ({ runBulkJob: mocks.runBulkJob }));
vi.mock('#lib/api/export.ts', async (original) => ({
  ...(await original<typeof import('#lib/api/export.ts')>()),
  downloadStoredExport: mocks.downloadStoredExport,
  uploadToImmich: mocks.uploadToImmich
}));
vi.mock('#lib/api/edits.ts', () => ({ getEdits: mocks.getEdits }));
vi.mock('#lib/panels/export/presetExport.ts', () => ({
  submitPresetExport: mocks.submitPresetExport
}));
vi.mock('#lib/utils/download.ts', () => ({ downloadBlob: mocks.downloadBlob }));
vi.mock('#lib/stores/jobs.svelte.ts', () => ({
  jobs: { showJob: mocks.showJob, downloadWhenDone: mocks.downloadWhenDone }
}));

import type { Job } from '#lib/api/jobs.ts';
import { BulkExporter, submitBulkExport } from '#lib/panels/export/bulkExport.svelte.ts';
import type { ExportPreset } from '#lib/api/exportPresets.ts';
import { exportPresets } from '#lib/panels/export/exportPresets.svelte.ts';
import { exportSettings } from '#lib/panels/export/exportSettings.svelte.ts';
import { exportScope, uploadOptions } from '#lib/panels/export/scope.ts';
import { baseOptions } from '#lib/panels/export/settings.ts';
import { immichCapabilities } from '#lib/stores/immichCapabilities.svelte.ts';
import { selection } from '#lib/stores/selection.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { mdiCloudUpload, mdiDownload, mdiFolderZip } from '@mdi/js';

const OPEN_SCOPE = exportScope({ assets: [], count: 2, trashOn: true });

const JOB: Job = {
  id: 'job-1',
  kind: 'download_zip',
  status: 'pending',
  target: {},
  params: {},
  total: 2,
  completed: 0,
  failed: 0,
  cancelled_at: null,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z'
};

beforeEach(() => {
  mocks.runBulkJob.mockImplementation(async (create: (ids: string[]) => Promise<unknown>) => {
    await create(['a', 'b']);
    return JOB;
  });
});

function preset(id: string, destination: ExportPreset['destination']): ExportPreset {
  return {
    id,
    name: id,
    form: {},
    destination,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z'
  };
}

afterEach(() => {
  toasts.items = [];
  exportPresets.presets = [];
  selection.clear();
  exportSettings.destination = 'download';
  immichCapabilities.trash = false;
  vi.resetAllMocks();
});

describe('submitBulkExport', () => {
  it('queues a ZIP download', async () => {
    const form = exportSettings.form;
    await expect(
      submitBulkExport({ form, destination: 'download', scope: OPEN_SCOPE, replaceOriginal: true })
    ).resolves.toBe(JOB);
    expect(mocks.createZipExportJob).toHaveBeenCalledWith(['a', 'b'], baseOptions(form));
    expect(mocks.createImmichExportJob).not.toHaveBeenCalled();
    expect(mocks.downloadWhenDone).toHaveBeenCalledWith('job-1');
  });

  it('queues an Immich export with the replace rule applied', async () => {
    const form = exportSettings.form;
    await submitBulkExport({
      form,
      destination: 'immich',
      scope: OPEN_SCOPE,
      replaceOriginal: true
    });
    expect(mocks.createImmichExportJob).toHaveBeenCalledWith(
      ['a', 'b'],
      uploadOptions(form, OPEN_SCOPE, true)
    );
    expect(mocks.downloadWhenDone).not.toHaveBeenCalled();
  });

  it('queues an Immich export with the earlier-export choice', async () => {
    const form = exportSettings.form;
    await submitBulkExport({
      form,
      destination: 'immich',
      scope: OPEN_SCOPE,
      replaceOriginal: false,
      onConflict: 'skip'
    });
    expect(mocks.createImmichExportJob).toHaveBeenCalledWith(
      ['a', 'b'],
      uploadOptions(form, OPEN_SCOPE, false, 'skip')
    );
  });
});

describe('BulkExporter', () => {
  it('labels the button by destination and selection', () => {
    const exporter = new BulkExporter(() => {});
    expect(exporter.disabled).toBe(true);

    selection.toggle('a');
    expect(exporter.label).toBe('Download JPEG');
    expect(exporter.icon).toBe(mdiDownload);

    selection.toggle('b');
    expect(exporter.label).toBe('Download 2 as JPEG ZIP');
    expect(exporter.icon).toBe(mdiFolderZip);
    expect(exporter.disabled).toBe(false);

    exportSettings.destination = 'immich';
    expect(exporter.label).toBe('Export 2 to Immich');
    expect(exporter.icon).toBe(mdiCloudUpload);
  });

  it('asks about earlier uploads of the selection and of its Immich presets apart', () => {
    exportPresets.presets = [preset('web', 'download'), preset('family', 'immich')];
    const exporter = new BulkExporter(() => {});
    expect(exporter.earlierQuery).toBeNull();
    selection.replace(['a', 'b']);
    expect(exporter.earlierQuery).toBeNull();
    exportSettings.destination = 'immich';
    expect(exporter.earlierQuery).toEqual({ assetIds: ['a', 'b'], presets: [] });
    exporter.presets.toggle('web', true);
    expect(exporter.presetQuery).toBeNull();
    exporter.presets.toggle('family', true);
    expect(exporter.presetQuery).toEqual({ assetIds: ['a', 'b'], presets: ['family'] });
    expect(exporter.earlierQuery).toEqual({ assetIds: ['a', 'b'], presets: [] });
  });

  it('sends the earlier-export choice with the form', async () => {
    selection.replace(['a', 'b']);
    exportSettings.destination = 'immich';
    const exporter = new BulkExporter(() => {});
    vi.spyOn(exporter.earlier, 'take').mockReturnValue('skip');
    await exporter.submit();
    expect(mocks.createImmichExportJob).toHaveBeenCalledWith(
      ['a', 'b'],
      expect.objectContaining({ onConflict: 'skip' })
    );
  });

  it('submits once, passes replace original and resets it', async () => {
    selection.replace(['a', 'b']);
    exportSettings.destination = 'immich';
    immichCapabilities.trash = true;
    let finish: (job: Job | null) => void = () => {};
    let queue: (ids: string[]) => Promise<unknown> = () => Promise.resolve();
    mocks.runBulkJob.mockImplementation((create: (ids: string[]) => Promise<unknown>) => {
      queue = create;
      return new Promise<Job | null>((resolve) => (finish = resolve));
    });
    const exporter = new BulkExporter(() => {});
    exporter.replaceOriginal = true;

    const submitted = exporter.submit();
    expect(exporter.busy).toBe(true);
    expect(exporter.disabled).toBe(true);
    expect(exporter.replaceOriginal).toBe(false);
    await exporter.submit();
    expect(mocks.runBulkJob).toHaveBeenCalledTimes(1);
    await queue(['a', 'b']);
    expect(mocks.createImmichExportJob).toHaveBeenCalledWith(
      ['a', 'b'],
      expect.objectContaining({ replaceOriginal: true })
    );

    finish(JOB);
    await submitted;
    expect(exporter.busy).toBe(false);
  });

  it('closes its container and opens the job once it is queued', async () => {
    selection.replace(['a', 'b']);
    const started = vi.fn();
    const exporter = new BulkExporter(started);

    await exporter.submit();
    expect(started).toHaveBeenCalledOnce();
    expect(mocks.showJob).toHaveBeenCalledWith('job-1');

    mocks.runBulkJob.mockResolvedValue(null);
    await exporter.submit();
    expect(started).toHaveBeenCalledOnce();
    expect(mocks.showJob).toHaveBeenCalledOnce();
  });

  it('uploads a single photo to Immich directly with its stored edits', async () => {
    selection.replace(['a']);
    exportSettings.destination = 'immich';
    mocks.getEdits.mockResolvedValue({ manifest: { version: 5, ops: {} } });
    mocks.uploadToImmich.mockResolvedValue({
      status: 'uploaded',
      filename: 'IMG_1_edit.jpg',
      warnings: []
    });
    const started = vi.fn();

    await new BulkExporter(started).submit();
    expect(mocks.uploadToImmich).toHaveBeenCalledWith(
      'a',
      expect.any(Object),
      expect.objectContaining({ onConflict: 'keep_both' })
    );
    expect(mocks.runBulkJob).not.toHaveBeenCalled();
    expect(started).toHaveBeenCalledOnce();
    expect(mocks.showJob).not.toHaveBeenCalled();
    expect(toasts.items.map((toast) => toast.message)).toEqual([
      'Uploaded IMG_1_edit.jpg to Immich'
    ]);
  });

  it('downloads a single photo directly with the form settings', async () => {
    selection.replace(['a']);
    const blob = new Blob(['jpg']);
    mocks.downloadStoredExport.mockResolvedValue({
      blob,
      filename: 'IMG_1_edit.jpg',
      warnings: []
    });
    const started = vi.fn();

    await new BulkExporter(started).submit();
    expect(mocks.downloadStoredExport).toHaveBeenCalledWith('a', baseOptions(exportSettings.form));
    expect(mocks.downloadBlob).toHaveBeenCalledWith(blob, 'IMG_1_edit.jpg');
    expect(mocks.runBulkJob).not.toHaveBeenCalled();
    expect(started).toHaveBeenCalledOnce();
    expect(toasts.items.map((toast) => toast.message)).toEqual(['Saved IMG_1_edit.jpg']);
  });

  it('keeps the sheet open when a single download fails', async () => {
    selection.replace(['a']);
    mocks.downloadStoredExport.mockRejectedValue(new Error('boom'));
    const started = vi.fn();
    const exporter = new BulkExporter(started);

    await exporter.submit();
    expect(started).not.toHaveBeenCalled();
    expect(exporter.busy).toBe(false);
    expect(toasts.items.map((toast) => toast.message)).toEqual(['Export failed: boom']);
  });

  it('keeps the form run apart from ticked presets', () => {
    exportPresets.presets = [preset('web', 'download')];
    selection.replace(['a', 'b']);
    const exporter = new BulkExporter(() => {});
    exporter.presets.toggle('web', true);
    expect(exporter.label).toBe('Download 2 as JPEG ZIP');
    expect(exporter.summary).toBe('2 photos · ZIP');
  });

  it('exports ticked presets from the preset menu, whatever the form says', async () => {
    exportPresets.presets = [preset('web', 'download'), preset('family', 'immich')];
    selection.replace(['a', 'b']);
    const started = vi.fn();
    const exporter = new BulkExporter(started);
    exporter.presets.toggle('web', true);
    exporter.presets.toggle('family', true);
    exportSettings.form.filenameTemplate = '{bad}';
    expect(exporter.disabled).toBe(true);
    vi.spyOn(exporter.earlierPresets, 'take').mockReturnValue('skip');
    mocks.submitPresetExport.mockResolvedValue([{ destination: 'immich', job: JOB }]);

    expect(await exporter.exportPresets()).toBe(false);
    expect(mocks.submitPresetExport).toHaveBeenCalledWith(
      ['a', 'b'],
      exportPresets.presets,
      exporter.scope,
      'skip'
    );
    expect(mocks.runBulkJob).not.toHaveBeenCalled();
    expect(started).toHaveBeenCalledOnce();
    expect(mocks.showJob).toHaveBeenCalledWith('job-1');
    expect(exporter.presets.picked.map((preset) => preset.id)).toEqual(['web']);
    expect(exporter.exportingPresets).toBe(false);
    exportSettings.form.filenameTemplate = '{name}_edit';
  });

  it('keeps the preset menu open when no preset job starts', async () => {
    exportPresets.presets = [preset('web', 'download')];
    selection.replace(['a', 'b']);
    const started = vi.fn();
    const exporter = new BulkExporter(started);
    exporter.presets.toggle('web', true);
    mocks.submitPresetExport.mockResolvedValue([]);

    expect(await exporter.exportPresets()).toBe(false);
    expect(started).not.toHaveBeenCalled();
    expect(exporter.presets.picked).toHaveLength(1);
  });
});
