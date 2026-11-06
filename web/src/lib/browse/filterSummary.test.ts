import { describe, expect, it } from 'vitest';
import { filterSummary } from './filterSummary';
import { FILTER_SECTIONS, type FilterSectionId } from '#lib/browse/filterSections.ts';
import { FILTER_DEFAULTS, type BrowseFilters } from '#lib/stores/browseControls.svelte.ts';

const NAMES: Record<string, string> = { a1: 'Summer', p1: 'Ann', p2: 'Bo' };

function summaryOf(
  id: FilterSectionId,
  filters: Partial<BrowseFilters> = {},
  supports?: (key: keyof BrowseFilters) => boolean
): string {
  const section = FILTER_SECTIONS.find((candidate) => candidate.id === id);
  if (!section) throw new Error(`no section ${id}`);
  return filterSummary(
    { ...FILTER_DEFAULTS, ...filters },
    section,
    (_, key) => NAMES[key] ?? 'Unknown',
    supports
  );
}

describe('filter section summaries', () => {
  it.each([
    ['library', 'Timeline · rejected shown'],
    ['collections', 'Any'],
    ['rating', 'Any rating · any label'],
    ['date', 'Any date'],
    ['camera', 'Any camera'],
    ['place', 'Any place'],
    ['file', 'Any name or size']
  ] as const)('describes the default %s section as %s', (id, summary) => {
    expect(summaryOf(id)).toBe(summary);
  });

  it('describes every section it knows', () => {
    for (const section of FILTER_SECTIONS) expect(summaryOf(section.id)).not.toBe('');
  });

  it('leaves unsupported controls out of a default summary', () => {
    expect(summaryOf('library', {}, (key) => key !== 'visibility')).toBe('Rejected shown');
    expect(summaryOf('library', {}, () => false)).toBe('Any');
  });

  it.each<[FilterSectionId, Partial<BrowseFilters>, string]>([
    ['library', { visibility: 'archive' }, 'Archived'],
    ['library', { rejected: 'hide', favoriteOnly: true }, 'No rejected · Favorites'],
    ['collections', { albums: { ids: ['a1'], match: 'all' } }, 'Albums: Summer'],
    ['collections', { noPeople: true }, 'No people'],
    ['rating', { rating: '3+' }, '≥ 3 stars'],
    ['rating', { label: 'none' }, 'No label'],
    ['date', { takenAfter: '2024-01-01' }, 'After 2024-01-01'],
    ['camera', { model: ['X-T5'] }, 'Model: X-T5'],
    ['place', { country: ['Norway'] }, 'Country: Norway'],
    ['file', { filename: 'DSC' }, 'Name: DSC']
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
    ).toBe('Albums: Summer · Without people: Ann +1 · Any of tags: Unknown · Untagged');
    expect(summaryOf('rating', { rating: 4, label: 'red' })).toBe('4 stars · Red label');
    expect(summaryOf('camera', { make: ['FUJIFILM'], lensModel: ['XF35', 'XF56'] })).toBe(
      'Make: FUJIFILM · Lens: XF35 +1'
    );
    expect(summaryOf('file', { minSize: 25, maxSize: 100 })).toBe('≥ 25 MB · ≤ 100 MB');
  });

  it('ignores filters that belong to other sections', () => {
    expect(summaryOf('camera', { country: ['Norway'], rating: 5 })).toBe('Any camera');
  });
});
