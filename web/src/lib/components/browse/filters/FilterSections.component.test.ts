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
import FilterPanel from './FilterPanel.svelte';
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
          FilterPanel,
          { favoriteLocked: locked, sortBasis: 'capture' },
          { exclude: [button('Reset all filters'), button('Close filter panel')] }
        ),
        layout(
          'sheet',
          FilterSheet,
          { favoriteLocked: locked, count: 3, sortBasis: 'capture' },
          {
            exclude: [
              button('Reset all filters'),
              button('Close view & filters'),
              button('Show 3 photos'),
              button('View')
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

describe('FilterSheet view section', () => {
  it('leads with View, summarised while collapsed and open, holding the view options when open', async () => {
    const destroy = renderLayout(layout('sheet', FilterSheet, { count: 3, sortBasis: 'capture' }));
    try {
      await settle();
      const triggers = [...document.querySelectorAll('[role="dialog"] [aria-expanded]')];
      const view = triggers.find((element) => element.getAttribute('aria-label') === 'View');
      expect(view).toBe(triggers[0]);
      const summary = document.getElementById(view?.getAttribute('aria-describedby') ?? '');
      expect(summary?.textContent.trim()).toMatch(/^Date taken, newest · (S|M|L|XL) · Badges$/);
      expect(names()).not.toContain('XL');

      browseView.setFilterSection('view', true);
      await settle();
      expect(view?.getAttribute('aria-describedby')).toBe(summary?.id);
      expect(names()).toEqual(
        expect.arrayContaining(['Date taken', 'Newest first', 'XL', 'Name and date'])
      );
    } finally {
      await destroy();
    }
  });
});

function names(): string[] {
  return controls(document.body).map((control) => control.name);
}
