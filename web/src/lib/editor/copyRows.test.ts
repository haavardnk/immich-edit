import { neutralEdits } from '#lib/types/edits.ts';
import { describe, expect, it } from 'vitest';
import { copyHints, rowDot, rowNote, splitColumns, visibleRows } from './copyRows';
import { COPY_TOOLS, DEVELOP_ROWS, PHOTO_ROWS, type CopyToolRow } from './copyTools';

function row(key: string, rows: readonly CopyToolRow[] = [...DEVELOP_ROWS, ...PHOTO_ROWS]) {
  const found = rows.find((r) => r.key === key);
  if (!found) throw new Error(key);
  return found;
}

describe('splitColumns', () => {
  it('balances the develop panels across two columns as in the mock', () => {
    const [left, right] = splitColumns(DEVELOP_ROWS, PHOTO_ROWS);
    expect(left.map((r) => r.key)).toEqual([
      'dcp',
      'basic',
      'curves',
      'hsl',
      'color-grading',
      'bw',
      'lut'
    ]);
    expect(right.map((r) => r.key)).toEqual(['detail', 'lens', 'lens-blur', 'effects']);
  });

  it('keeps the first panel beside Develop when few tools are on offer', () => {
    const rows = visibleRows(DEVELOP_ROWS, ['white_balance', 'tone', 'curves', 'hsl']);
    const [left, right] = splitColumns(rows, []);
    expect(left.map((r) => r.key)).toEqual(['basic']);
    expect(right.map((r) => r.key)).toEqual(['curves', 'hsl']);
  });
});

describe('visibleRows', () => {
  it('keeps only the tools on offer and the parents that hold them', () => {
    const rows = visibleRows(DEVELOP_ROWS, ['tone', 'hsl']);
    expect(rows.map((r) => [r.key, r.tools, r.children.map((c) => c.key)])).toEqual([
      ['basic', ['tone'], ['tone']],
      ['hsl', ['hsl'], []]
    ]);
  });

  it('keeps every row when every tool is on offer', () => {
    expect(visibleRows(DEVELOP_ROWS, COPY_TOOLS)).toEqual(DEVELOP_ROWS);
  });
});

describe('row hints', () => {
  const edits = neutralEdits();
  edits.color.hsl.bands[0] = { hue: 10, sat: 0, lum: 0 };
  edits.basic.exposure_ev = 1;
  edits.masks = [];
  edits.retouch = [];
  const hints = copyHints(edits, 'hsl');

  it('marks modified panels with the dot and the rest as not modified', () => {
    expect(rowDot(row('basic'), hints, false)).toBe(true);
    expect(rowNote(row('basic'), hints, false)).toBeNull();
    expect(rowDot(row('curves'), hints, false)).toBe(false);
    expect(rowNote(row('curves'), hints, false)).toBe('not modified');
  });

  it('names the last touched tool', () => {
    expect(rowNote(row('hsl'), hints, false)).toBe('last touched');
  });

  it('leaves nested sections quiet unless last touched', () => {
    const tone = row('tone', row('basic').children);
    const wb = row('white_balance', row('basic').children);
    expect(rowNote(wb, hints, true)).toBeNull();
    expect(rowDot(tone, hints, true)).toBe(false);
    expect(rowNote(tone, copyHints(edits, 'tone'), true)).toBe('last touched');
  });

  it('counts mask layers and retouch strokes when there are any', () => {
    expect(rowNote(row('masks'), hints, false)).toBeNull();
    expect(rowNote(row('retouch'), { ...hints, retouch: 12 }, false)).toBe('12 strokes');
    expect(rowNote(row('masks'), { ...hints, masks: 1 }, false)).toBe('1 layer');
    expect(rowNote(row('geometry'), hints, false)).toBeNull();
  });
});
