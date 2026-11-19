import { FILTER_SECTIONS } from '#lib/browse/filterSections.ts';
import { GRID_SIZE_OPTIONS, TILE_INFO_OPTIONS } from '#lib/browse/viewOptions.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import {
  controls,
  layout,
  layoutControls,
  renderLayout,
  settle,
  type ControlRef
} from '#lib/testing/layoutParity.ts';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import FilterPanel from './FilterPanel.svelte';
import FilterSections from './FilterSections.svelte';
import FilterSheet from './FilterSheet.svelte';

vi.mock('$app/navigation', () => ({ afterNavigate: (): void => {}, goto: vi.fn() }));

vi.mock('$app/state', () => ({
  page: { url: new URL('http://localhost/photos'), params: {} }
}));

const button = (name: string): ControlRef => ({ role: 'button', name });
const radio = (name: string): ControlRef => ({ role: 'radio', name });

const VIEW_BLOCK = [
  ...['Taken', 'Added', 'Newest', 'Oldest'],
  ...GRID_SIZE_OPTIONS.map((option) => option.label),
  ...TILE_INFO_OPTIONS.map((option) => option.label)
].map(radio);

beforeEach(() => {
  vi.stubGlobal(
    'fetch',
    vi.fn(() => Promise.reject(new Error('offline')))
  );
  browseView.setFiltersOpen(true);
});

afterEach(() => {
  browseView.setFiltersOpen(false);
  browseView.filterSections = [];
  vi.unstubAllGlobals();
});

describe('FilterSections layout parity', () => {
  const expanded = FILTER_SECTIONS.map((section) => section.id);

  it.each([
    ['collapsed', [], false],
    ['expanded', expanded, false],
    ['expanded with favourites locked', expanded, true]
  ])(
    'exposes the same controls standalone and in the filter panel (%s)',
    async (_, open, locked) => {
      browseView.filterSections = open;
      const found = await layoutControls([
        layout('body', FilterSections, { favoriteLocked: locked }),
        layout(
          'panel',
          FilterPanel,
          { favoriteLocked: locked, sortBasis: 'capture', title: 'Photos' },
          {
            exclude: [
              button('Save view'),
              button('Reset all filters'),
              button('Close filter panel')
            ]
          }
        ),
        layout(
          'sheet',
          FilterSheet,
          { favoriteLocked: locked, count: 3, sortBasis: 'capture', title: 'Photos' },
          {
            exclude: [
              button('Save view'),
              button('Reset all filters'),
              button('Close view & filters'),
              button('Show 3 photos'),
              ...VIEW_BLOCK
            ]
          }
        )
      ]);
      expect(found.body.length).toBeGreaterThanOrEqual(FILTER_SECTIONS.length);
      expect(found.panel).toEqual(found.body);
      expect(found.sheet).toEqual(found.body);
      const favorites = found.body.some((control) => control.name === 'Favorites only');
      expect(favorites).toBe(open.length > 0 && !locked);
    }
  );
});

describe('FilterSheet view block', () => {
  it('leads with sort, order, size and thumbnail info above the filters', async () => {
    const destroy = renderLayout(
      layout('sheet', FilterSheet, { count: 3, sortBasis: 'capture', title: 'Photos' })
    );
    try {
      await settle();
      const groups = [...document.querySelectorAll('[role="dialog"] [role="radiogroup"]')].map(
        (group) => group.getAttribute('aria-label')
      );
      expect(groups.slice(0, 4)).toEqual(['Sort by', 'Order', 'Thumbnail size', 'Thumbnail info']);
      expect(names()).toEqual(
        expect.arrayContaining(['Taken', 'Newest', 'XL', 'Name and date', 'Reset all filters'])
      );
      expect(names()).not.toContain('View');
    } finally {
      await destroy();
    }
  });
});

function names(): string[] {
  return controls(document.body).map((control) => control.name);
}
