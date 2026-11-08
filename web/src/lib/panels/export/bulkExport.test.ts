import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  createImmichExportJob: vi.fn(),
  createZipExportJob: vi.fn(),
  runBulkJob: vi.fn(),
  showJob: vi.fn(),
  downloadWhenDone: vi.fn(),
  downloadStoredExport: vi.fn(),
  downloadBlob: vi.fn()
}));

vi.mock('#lib/api/jobs.ts', async (original) => ({
  ...(await original<typeof import('#lib/api/jobs.ts')>()),
  createImmichExportJob: mocks.createImmichExportJob,
  createZipExportJob: mocks.createZipExportJob
}));
vi.mock('#lib/api/bulkJob.ts', () => ({ runBulkJob: mocks.runBulkJob }));
vi.mock('#lib/api/export.ts', async (original) => ({
  ...(await original<typeof import('#lib/api/export.ts')>()),
  downloadStoredExport: mocks.downloadStoredExport
}));
vi.mock('#lib/utils/download.ts', () => ({ downloadBlob: mocks.downloadBlob }));
vi.mock('#lib/stores/jobs.svelte.ts', () => ({
  jobs: { showJob: mocks.showJob, downloadWhenDone: mocks.downloadWhenDone }
}));

import type { Job } from '#lib/api/jobs.ts';
import { BulkExporter, submitBulkExport } from '#lib/panels/export/bulkExport.svelte.ts';
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

afterEach(() => {
  toasts.items = [];
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
});

describe('BulkExporter', () => {
  it('labels the button by destination and selection', () => {
    const exporter = new BulkExporter(() => {});
    expect(exporter.disabled).toBe(true);

    selection.toggle('a');
    expect(exporter.label).toBe('Download JPEG');
    expect(exporter.icon).toBe(mdiDownload);
    expect(exporter.downloadLabel).toBe('Download');

    selection.toggle('b');
    expect(exporter.label).toBe('Download 2 as JPEG ZIP');
    expect(exporter.icon).toBe(mdiFolderZip);
    expect(exporter.downloadLabel).toBe('Download ZIP');
    expect(exporter.disabled).toBe(false);

    exportSettings.destination = 'immich';
    expect(exporter.label).toBe('Export 2 to Immich');
    expect(exporter.icon).toBe(mdiCloudUpload);
  });

  it('submits once, passes replace original and resets it', async () => {
    selection.toggle('a');
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
    await queue(['a']);
    expect(mocks.createImmichExportJob).toHaveBeenCalledWith(
      ['a'],
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
});
