import { afterEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  createImmichPresetsJob: vi.fn(),
  createZipPresetsJob: vi.fn(),
  downloadWhenDone: vi.fn(),
  track: vi.fn()
}));

vi.mock('#lib/api/jobs.ts', async (original) => ({
  ...(await original<typeof import('#lib/api/jobs.ts')>()),
  createImmichPresetsJob: mocks.createImmichPresetsJob,
  createZipPresetsJob: mocks.createZipPresetsJob
}));
vi.mock('#lib/stores/jobs.svelte.ts', () => ({
  jobs: { downloadWhenDone: mocks.downloadWhenDone, track: mocks.track }
}));

import type { ExportPreset } from '#lib/api/exportPresets.ts';
import { presetVariants, submitPresetExport } from './presetExport';
import { exportScope } from './scope';
import { toasts } from '#lib/stores/toasts.svelte.ts';

const SCOPE = exportScope({ assets: [], count: 1, trashOn: true });

function preset(name: string, destination: ExportPreset['destination'], form = {}): ExportPreset {
  return {
    id: name,
    name,
    form,
    destination,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z'
  };
}

afterEach(() => {
  toasts.items = [];
  vi.resetAllMocks();
});

describe('preset export', () => {
  it('splits presets by destination and builds each from its own form', () => {
    const { zip, immich } = presetVariants(
      [
        preset('Web', 'download', { format: 'webp' }),
        preset('Family', 'immich', { albumIds: ['al'] }),
        preset('Print', 'download', { format: 'tiff', filenameTemplate: '{name}_print' })
      ],
      SCOPE
    );
    expect(zip.map((v) => [v.preset, v.options.format, v.options.filenameTemplate])).toEqual([
      ['Web', 'webp', '{name}_{preset}'],
      ['Print', 'tiff', '{name}_print']
    ]);
    expect(immich).toHaveLength(1);
    expect(immich[0]?.options).toMatchObject({
      albumIds: ['al'],
      replaceOriginal: false,
      onConflict: 'keep_both'
    });
  });

  it('applies the earlier-export choice to every Immich preset', () => {
    const { immich } = presetVariants(
      [preset('Family', 'immich'), preset('Share', 'immich')],
      SCOPE,
      'skip'
    );
    expect(immich.map((v) => v.options.onConflict)).toEqual(['skip', 'skip']);
  });

  it('queues one job per destination and downloads the ZIP when done', async () => {
    mocks.createZipPresetsJob.mockResolvedValue({ id: 'zip', status: 'pending' });
    mocks.createImmichPresetsJob.mockResolvedValue({ id: 'up', status: 'pending' });
    const started = await submitPresetExport(
      ['a'],
      [preset('Web', 'download'), preset('Family', 'immich'), preset('Print', 'download')],
      SCOPE
    );
    expect(started.map(({ destination, job }) => [destination, job.id])).toEqual([
      ['download', 'zip'],
      ['immich', 'up']
    ]);
    expect(mocks.createZipPresetsJob).toHaveBeenCalledWith(
      ['a'],
      [expect.objectContaining({ preset: 'Web' }), expect.objectContaining({ preset: 'Print' })]
    );
    expect(mocks.downloadWhenDone).toHaveBeenCalledWith('zip');
    expect(mocks.downloadWhenDone).toHaveBeenCalledTimes(1);
  });

  it('keeps the jobs that started when another fails', async () => {
    mocks.createZipPresetsJob.mockRejectedValue(new Error('bad name'));
    mocks.createImmichPresetsJob.mockResolvedValue({ id: 'up', status: 'pending' });
    const started = await submitPresetExport(
      ['a'],
      [preset('Web', 'download'), preset('Family', 'immich')],
      SCOPE
    );
    expect(started.map(({ job }) => job.id)).toEqual(['up']);
    expect(toasts.items.map((toast) => toast.message)).toEqual([
      'Failed to queue export: bad name'
    ]);
  });
});
