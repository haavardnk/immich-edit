import { describe, expect, it } from 'vitest';
import {
  runDirect,
  runFiles,
  runLabel,
  runNote,
  runSummary,
  runZips,
  type ExportRun
} from './exportRun';

function run(patch: Partial<ExportRun> = {}): ExportRun {
  return {
    photos: 124,
    presets: [],
    destination: 'download',
    format: 'JPEG',
    albums: [],
    ...patch
  };
}

describe('export run', () => {
  it('labels current settings by destination and count', () => {
    expect(runLabel(run({ photos: 1 }))).toBe('Download JPEG');
    expect(runLabel(run())).toBe('Download 124 as JPEG ZIP');
    expect(runLabel(run({ destination: 'immich' }))).toBe('Export 124 to Immich');
  });

  it('counts files when presets are ticked', () => {
    const immich = run({ presets: ['immich', 'immich'] });
    expect(runFiles(immich)).toBe(248);
    expect(runLabel(immich)).toBe('Export 248 to Immich');
    expect(runLabel(run({ presets: ['download', 'download'] }))).toBe('Download 248 as ZIP');
    expect(runLabel(run({ presets: ['download', 'immich'] }))).toBe('Export 248 files');
  });

  it('sums up photos, presets and where they go', () => {
    expect(runSummary(run({ destination: 'immich', albums: ['Bergen 2026'] }))).toBe(
      '124 photos · to Immich album Bergen 2026'
    );
    expect(runSummary(run({ destination: 'immich', albums: ['A', 'B'] }))).toBe(
      '124 photos · to Immich, 2 albums'
    );
    expect(runSummary(run({ presets: ['immich', 'immich'] }))).toBe(
      '124 photos · 2 presets · to Immich'
    );
    expect(runSummary(run({ photos: 1, presets: ['download', 'immich'] }))).toBe(
      '1 photo · 2 presets · ZIP and Immich'
    );
    expect(runSummary(run({ photos: 1 }))).toBe('1 photo · download');
  });

  it('says how many jobs run, except for a direct download', () => {
    expect(runNote(run({ photos: 1 }))).toBeNull();
    expect(runNote(run())).toBe('Runs as a background job. Track progress in Jobs.');
    expect(runNote(run({ presets: ['download', 'immich'] }))).toBe(
      'Runs as 2 background jobs. Track progress in Jobs.'
    );
  });

  it('knows when the result is a ZIP', () => {
    expect(runDirect(run({ photos: 1 }))).toBe(true);
    expect(runZips(run({ photos: 1 }))).toBe(false);
    expect(runZips(run())).toBe(true);
    expect(runZips(run({ destination: 'immich' }))).toBe(false);
    expect(runZips(run({ photos: 1, presets: ['download'] }))).toBe(true);
    expect(runZips(run({ presets: ['immich'] }))).toBe(false);
  });
});
