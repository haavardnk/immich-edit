import { describe, expect, it } from 'vitest';
import { FILTER_SECTIONS, isFilterSectionId } from './filterSections';
import { FILTER_DEFAULTS } from '#lib/stores/browseControls.svelte.ts';

describe('filter panel sections', () => {
  it('place every filter in exactly one section', () => {
    const keys = FILTER_SECTIONS.flatMap((s) => s.keys);
    expect(new Set(keys).size).toBe(keys.length);
    expect([...keys].sort()).toEqual(Object.keys(FILTER_DEFAULTS).sort());
  });

  it('knows only the filter sections', () => {
    expect(isFilterSectionId('camera')).toBe(true);
    expect(isFilterSectionId('view')).toBe(false);
    expect(isFilterSectionId('lens')).toBe(false);
  });
});
