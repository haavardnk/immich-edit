import type { ExportedFile } from '#lib/api/exportOutputs.ts';
import { describe, expect, it } from 'vitest';
import {
  earlierChoices,
  earlierFiles,
  earlierHint,
  earlierTitle,
  type EarlierRun
} from './earlierExport';

const NOW = new Date('2026-11-15T12:00:00Z');

const file = (filename: string, exported_at = '2026-11-15T11:40:00Z'): ExportedFile => ({
  filename,
  exported_at
});

function run(total: number, photos: number, files: ExportedFile[] = [], presets = 0): EarlierRun {
  return { found: { photos, files }, total, presets };
}

const labels = (r: EarlierRun): string[] => earlierChoices(r).map((choice) => choice.label);

describe('earlier export choices', () => {
  it('offers Replace it or Keep both for one earlier file', () => {
    expect(labels(run(1, 1, [file('a_edit.jpg')]))).toEqual(['Replace it', 'Keep both']);
  });

  it('offers Replace them for one photo exported with several presets', () => {
    const files = [file('a_web.jpg'), file('a_print.jpg')];
    expect(labels(run(1, 1, files, 2))).toEqual(['Replace them', 'Keep both']);
  });

  it('adds Skip them for several photos', () => {
    expect(labels(run(124, 38))).toEqual(['Replace them', 'Skip them', 'Keep both']);
    expect(earlierChoices(run(124, 38)).map((choice) => choice.value)).toEqual([
      'replace',
      'skip',
      'keep_both'
    ]);
  });
});

describe('earlierTitle', () => {
  it.each<[EarlierRun, string]>([
    [run(1, 1, [file('a.jpg')]), 'Exported to Immich before'],
    [run(1, 1, [file('a.jpg')], 1), 'Exported to Immich before with this preset'],
    [run(124, 38), '38 of 124 photos were exported to Immich before'],
    [run(124, 1), '1 of 124 photos was exported to Immich before'],
    [run(3, 3, [], 2), 'All 3 photos were exported to Immich before with these presets']
  ])('describes %j', (r, title) => {
    expect(earlierTitle(r)).toBe(title);
  });
});

describe('earlierFiles', () => {
  it('names the latest file and when it was exported', () => {
    expect(earlierFiles([file('a_edit.jpg')], NOW)).toBe('a_edit.jpg · 20 min ago');
  });

  it('counts the other files', () => {
    expect(earlierFiles([file('a_web.jpg'), file('a_print.jpg')], NOW)).toBe(
      'a_web.jpg · 20 min ago and 1 more'
    );
  });

  it('is empty without files', () => {
    expect(earlierFiles([], NOW)).toBeNull();
  });
});

describe('earlierHint', () => {
  const one = run(1, 1, [file('a_edit.jpg')]);
  const bulk = run(124, 38);

  it('says which file moves to the trash', () => {
    expect(earlierHint(one, 'replace')).toBe(
      'a_edit.jpg moves to the Immich trash; the new export takes its albums and stack.'
    );
  });

  it('says only the latest export of each photo is replaced', () => {
    expect(earlierHint(bulk, 'replace')).toBe(
      'The latest earlier export of each photo moves to the Immich trash; the new one takes its albums and stack.'
    );
    expect(earlierHint(run(1, 1, [file('a'), file('b')], 2), 'replace')).toContain(
      'each photo and preset'
    );
  });

  it('keeps both next to each other', () => {
    expect(earlierHint(one, 'keep_both')).toBe('The new export is added next to the earlier one.');
    expect(earlierHint(bulk, 'keep_both')).toBe(
      'The new exports are added next to the earlier ones.'
    );
  });

  it('counts what Skip still uploads', () => {
    expect(earlierHint(bulk, 'skip')).toBe('Only the other 86 photos are uploaded.');
    expect(earlierHint(run(2, 1), 'skip')).toBe('Only the other photo is uploaded.');
    expect(earlierHint(run(2, 2), 'skip')).toBe('Nothing is uploaded.');
  });
});
