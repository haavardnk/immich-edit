import { FILTER_SECTIONS } from '#lib/browse/filterSections.ts';
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
import FilterDock from './FilterDock.svelte';
import FilterSections from './FilterSections.svelte';
import FilterSheet from './FilterSheet.svelte';

vi.mock('$app/navigation', () => ({ afterNavigate: (): void => {}, goto: vi.fn() }));

vi.mock('$app/state', () => ({
  page: { url: new URL('http://localhost/photos'), params: {} }
}));

const button = (name: string): ControlRef => ({ role: 'button', name });

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
          FilterDock,
          { favoriteLocked: locked, title: 'Photos', onClose: () => {} },
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
          { favoriteLocked: locked, count: 3, title: 'Photos' },
          {
            exclude: [
              button('Save view'),
              button('Reset all filters'),
              button('Close filters'),
              button('Show 3 photos')
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

describe('FilterSheet', () => {
  it('holds only the filters, with Reset and Save view in its header', async () => {
    const destroy = renderLayout(layout('sheet', FilterSheet, { count: 3, title: 'Photos' }));
    try {
      await settle();
      const sheet = document.querySelector('[role="dialog"]');
      expect(
        document.getElementById(sheet?.getAttribute('aria-labelledby') ?? '')?.textContent?.trim()
      ).toBe('Filters');
      expect(sheet?.querySelectorAll('[role="radiogroup"]')).toHaveLength(0);
      expect(names()).toEqual(
        expect.arrayContaining(['Reset all filters', 'Save view', 'Close filters', 'Show 3 photos'])
      );
      expect(names()).not.toContain('Thumbnail size S');
    } finally {
      await destroy();
    }
  });
});

function names(): string[] {
  return controls(document.body).map((control) => control.name);
}
