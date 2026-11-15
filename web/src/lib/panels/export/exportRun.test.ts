import { describe, expect, it } from 'vitest';
import { runDirect, runLabel, runNote, runSummary, runZips, type ExportRun } from './exportRun';

function run(patch: Partial<ExportRun> = {}): ExportRun {
  return {
    photos: 124,
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

  it('sums up photos and where they go', () => {
    expect(runSummary(run({ destination: 'immich', albums: ['Bergen 2026'] }))).toBe(
      '124 photos · to Immich album Bergen 2026'
    );
    expect(runSummary(run({ destination: 'immich', albums: ['A', 'B'] }))).toBe(
      '124 photos · to Immich, 2 albums'
    );
    expect(runSummary(run())).toBe('124 photos · ZIP');
    expect(runSummary(run({ photos: 1 }))).toBe('1 photo · download');
  });

  it('says it runs as a job, except for a direct download', () => {
    expect(runNote(run({ photos: 1 }))).toBeNull();
    expect(runNote(run())).toBe('Runs as a background job. Track progress in Jobs.');
  });

  it('knows when the result is a ZIP', () => {
    expect(runDirect(run({ photos: 1 }))).toBe(true);
    expect(runZips(run({ photos: 1 }))).toBe(false);
    expect(runZips(run())).toBe(true);
    expect(runZips(run({ destination: 'immich' }))).toBe(false);
  });
});
