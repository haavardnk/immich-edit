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
import { neutralEdits } from '$lib/types/edits';

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
  it('excludes geometry and masks by default', () => {
    expect(DEFAULT_COPY_SECTIONS.geometry).toBe(false);
    expect(DEFAULT_COPY_SECTIONS.masks).toBe(false);
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
});
