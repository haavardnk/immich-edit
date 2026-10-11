import { describe, expect, it } from 'vitest';
import { FILTER_SECTIONS, sectionModified, withoutSection } from './filterSections';
import { FILTER_DEFAULTS, type BrowseFilters } from '$lib/stores/browseControls.svelte';

const section = (id: string) => {
  const found = FILTER_SECTIONS.find((s) => s.id === id);
  if (!found) throw new Error(`missing section ${id}`);
  return found;
};

describe('filter panel sections', () => {
  it('place every filter in exactly one section', () => {
    const keys = FILTER_SECTIONS.flatMap((s) => s.keys);
    expect(new Set(keys).size).toBe(keys.length);
    expect([...keys].sort()).toEqual(Object.keys(FILTER_DEFAULTS).sort());
  });

  it.each<[string, Partial<BrowseFilters>]>([
    ['library', { rejected: 'hide' }],
    ['rating', { label: 'none' }],
    ['date', { takenBefore: '2024-01-01' }],
    ['file', { filename: 'DSC' }],
    ['camera', { lensModel: ['FE 35mm F1.8'] }],
    ['place', { city: ['Bergen'] }]
  ])('marks %s modified only by its own filters', (id, change) => {
    const filters = { ...FILTER_DEFAULTS, ...change };

    expect(FILTER_SECTIONS.filter((s) => sectionModified(filters, s)).map((s) => s.id)).toEqual([
      id
    ]);
  });

  it('resets one section and keeps the others', () => {
    const filters: BrowseFilters = {
      ...FILTER_DEFAULTS,
      favoriteOnly: true,
      make: ['SONY'],
      model: ['ILCE-7M3'],
      country: ['Norway']
    };

    expect(withoutSection(filters, section('camera'))).toEqual({
      ...FILTER_DEFAULTS,
      favoriteOnly: true,
      country: ['Norway']
    });
  });
});
