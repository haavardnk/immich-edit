import { describe, expect, it } from 'vitest';
import { filterRows, PREVIEW_ROWS, type FilterListItem } from './filterList.ts';

const items: FilterListItem[] = [
  { id: 'a', label: 'Aksel', count: 3 },
  { id: 'b', label: 'Berit', count: 90 },
  { id: 'c', label: 'Cato', count: 40 },
  { id: 'd', label: 'Dina', count: null },
  { id: 'e', label: 'Eirik', count: 12 },
  { id: 'f', label: 'Frida', count: 12 },
  { id: 'g', label: 'Gunn', count: 7 }
];

const picked = (ids: string[]) => (id: string) => ids.includes(id);

describe('filterRows', () => {
  it('pins the ticked rows and previews the largest of the rest', () => {
    const rows = filterRows(items, picked(['g', 'a']), '', false);
    expect(rows.pinned.map((item) => item.id)).toEqual(['a', 'g']);
    expect(rows.rest.map((item) => item.id)).toEqual(['b', 'c', 'e', 'f', 'd']);
    expect(rows.rest).toHaveLength(PREVIEW_ROWS);
    expect(rows.hidden).toBe(false);
  });

  it('says more are hidden when the preview leaves rows out', () => {
    expect(filterRows(items, picked([]), '', false).hidden).toBe(true);
  });

  it('lists everything else A to Z when full', () => {
    const rows = filterRows(items, picked(['b']), '', true);
    expect(rows.rest.map((item) => item.label)).toEqual([
      'Aksel',
      'Cato',
      'Dina',
      'Eirik',
      'Frida',
      'Gunn'
    ]);
    expect(rows.hidden).toBe(false);
  });

  it('searches every row and keeps the ticked ones pinned', () => {
    const rows = filterRows(items, picked(['b']), 'i', false);
    expect(rows.pinned.map((item) => item.id)).toEqual(['b']);
    expect(rows.rest.map((item) => item.label)).toEqual(['Dina', 'Eirik', 'Frida']);
  });
});
