import { describe, expect, it } from 'vitest';
import { filterSummary } from './filterSummary';
import { FILTER_SECTIONS, type FilterSectionId } from '#lib/browse/filterSections.ts';
import { FILTER_DEFAULTS, type BrowseFilters } from '#lib/stores/browseControls.svelte.ts';

const NAMES: Record<string, string> = { a1: 'Summer', p1: 'Ann', p2: 'Bo' };

function summaryOf(id: FilterSectionId, filters: Partial<BrowseFilters> = {}): string | undefined {
  const section = FILTER_SECTIONS.find((candidate) => candidate.id === id);
  if (!section) throw new Error(`no section ${id}`);
  return filterSummary(
    { ...FILTER_DEFAULTS, ...filters },
    section,
    (_, key) => NAMES[key] ?? 'Unknown'
  );
}

describe('filter section summaries', () => {
  it('has no summary for an unmodified section', () => {
    for (const section of FILTER_SECTIONS) expect(summaryOf(section.id)).toBeUndefined();
  });

  it.each<[FilterSectionId, Partial<BrowseFilters>, string]>([
    ['library', { visibility: 'archive' }, 'Archived'],
    ['library', { rejected: 'hide', favoriteOnly: true }, 'No rejected · Favorites'],
    ['collections', { albums: { ids: ['a1'], match: 'all' } }, 'Summer'],
    ['collections', { noPeople: true }, 'No people'],
    ['rating', { rating: '3+' }, '≥ 3 stars'],
    ['rating', { label: 'none' }, 'No label'],
    ['date', { takenAfter: '2024-10-01', takenBefore: '2024-10-31' }, 'Oct 2024'],
    ['date', { addedAfter: '2024-01-01', addedBefore: '2024-12-31' }, 'Added 2024'],
    ['camera', { model: ['X-T5'] }, 'X-T5'],
    ['place', { country: ['Norway'], state: ['Vestland'] }, 'Norway · Vestland'],
    ['file', { filename: 'DSC' }, 'Name: DSC'],
    ['file', { minSize: 1, maxSize: 10 }, '1–10 MB']
  ])('summarises a modified %s section with %j as %s', (id, filters, summary) => {
    expect(summaryOf(id, filters)).toBe(summary);
  });

  it('joins several values of one section in panel order', () => {
    expect(
      summaryOf('collections', {
        tags: { ids: ['t1'], match: 'any' },
        people: { ids: ['p1', 'p2'], match: 'none' },
        albums: { ids: ['a1'], match: 'all' },
        untagged: true
      })
    ).toBe('Summer · Without people: Ann +1 · Any of tags: Unknown · Untagged');
    expect(summaryOf('rating', { rating: 4, label: 'red' })).toBe('4 stars · Red label');
    expect(summaryOf('camera', { make: ['FUJIFILM'], lensModel: ['XF35', 'XF56'] })).toBe(
      'FUJIFILM · XF35 +1'
    );
  });

  it('ignores filters that belong to other sections', () => {
    expect(summaryOf('camera', { country: ['Norway'], rating: 5 })).toBeUndefined();
  });
});
