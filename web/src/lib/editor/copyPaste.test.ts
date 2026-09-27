import { describe, it, expect } from 'vitest';
import {
  allSections,
  hasSelectedSections,
  allSelected,
  applyCopySections,
  DEFAULT_COPY_SECTIONS,
  DEVELOP_KEYS,
  type CopySections
} from './copyPaste';
import { neutralEdits, type RetouchStroke } from '$lib/types/edits';

function stroke(id: string): RetouchStroke {
  return {
    id,
    mode: 'heal',
    points: [{ x: 0.5, y: 0.5 }],
    radius: 0.02,
    hardness: 0.5,
    opacity: 1,
    source: { x: 0.6, y: 0.6 },
    enabled: true
  };
}

describe('allSections', () => {
  it('sets every section to the given value', () => {
    expect(Object.values(allSections(true)).every((v) => v)).toBe(true);
    expect(Object.values(allSections(false)).every((v) => !v)).toBe(true);
  });
});

describe('hasSelectedSections', () => {
  it('is false when nothing is selected', () => {
    expect(hasSelectedSections(allSections(false))).toBe(false);
  });

  it('is true when at least one section is selected', () => {
    expect(hasSelectedSections({ ...allSections(false), tone: true })).toBe(true);
  });
});

describe('allSelected', () => {
  it('checks every key in the subset', () => {
    expect(allSelected(allSections(true), DEVELOP_KEYS)).toBe(true);
    expect(allSelected({ ...allSections(true), tone: false }, DEVELOP_KEYS)).toBe(false);
  });
});

describe('DEFAULT_COPY_SECTIONS', () => {
  it('excludes geometry, masks and retouch by default', () => {
    expect(DEFAULT_COPY_SECTIONS.geometry).toBe(false);
    expect(DEFAULT_COPY_SECTIONS.masks).toBe(false);
    expect(DEFAULT_COPY_SECTIONS.retouch).toBe(false);
    expect(DEFAULT_COPY_SECTIONS.basic).toBe(true);
  });
});

describe('applyCopySections', () => {
  it('takes only the selected sections from incoming', () => {
    const current = neutralEdits();
    current.basic.exposure_ev = 1;
    current.tone.shadows = 5;
    const incoming = neutralEdits();
    incoming.basic.exposure_ev = 2;
    incoming.tone.shadows = 9;

    const sections: CopySections = { ...allSections(false), basic: true };
    const out = applyCopySections(current, incoming, sections);
    expect(out.basic.exposure_ev).toBe(2);
    expect(out.tone.shadows).toBe(5);
  });

  it('returns current sections when nothing is selected', () => {
    const current = neutralEdits();
    current.basic.contrast = 7;
    const incoming = neutralEdits();
    incoming.basic.contrast = 99;
    const out = applyCopySections(current, incoming, allSections(false));
    expect(out.basic.contrast).toBe(7);
  });

  it('carries heal and clone strokes only when retouch is selected', () => {
    const current = neutralEdits();
    current.retouch = [stroke('current')];
    const incoming = neutralEdits();
    incoming.retouch = [stroke('incoming')];

    const kept = applyCopySections(current, incoming, allSections(false));
    expect(kept.retouch).toEqual([stroke('current')]);

    const pasted = applyCopySections(current, incoming, {
      ...allSections(false),
      retouch: true
    });
    expect(pasted.retouch).toEqual([stroke('incoming')]);
  });
});
