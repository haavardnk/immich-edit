import { FILTER_SECTIONS } from '#lib/browse/filterSections.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { layout, layoutControls, type ControlRef } from '#lib/testing/layoutParity.ts';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import FilterPanel from './FilterPanel.svelte';
import FilterSections from './FilterSections.svelte';

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
          { favoriteLocked: locked },
          { exclude: [button('Reset all filters'), button('Close filter panel')] }
        )
      ]);
      expect(found.body.length).toBeGreaterThanOrEqual(FILTER_SECTIONS.length);
      expect(found.panel).toEqual(found.body);
      const favorites = found.body.some((control) => control.name === 'Favorites only');
      expect(favorites).toBe(open.length > 0 && !locked);
    }
  );
});
