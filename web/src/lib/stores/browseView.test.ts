import type { LayoutMode } from '#lib/device/layoutRules.ts';
import { afterEach, describe, expect, it, vi } from 'vitest';

const storageKey = 'immich-edit:browseView';

const layoutMock = await vi.hoisted(async () => import('#lib/testing/layoutMock.ts'));

vi.mock('#lib/device/layout.svelte.ts', () => ({ layout: layoutMock.fakeLayout }));

afterEach(() => {
  vi.unstubAllGlobals();
  vi.resetModules();
  layoutMock.resetLayout();
});

function onLayout(mode: LayoutMode, wide = false, phone = false): void {
  layoutMock.resetLayout({ mode, wide, phone });
}

function stubStorage(initial?: unknown): Map<string, string> {
  const values = new Map<string, string>();
  if (initial !== undefined) values.set(storageKey, JSON.stringify(initial));
  vi.stubGlobal('localStorage', {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => values.set(key, value)
  });
  return values;
}

describe('browse view persistence', () => {
  it('restores the stored grid size and loupe flags', async () => {
    stubStorage({
      gridSizeChoice: 'xl',
      loupeAutoAdvance: true,
      loupeInfoOpen: true,
      tileInfo: 'full'
    });

    const { browseView } = await import('./browseView.svelte');

    expect(browseView.gridSize).toBe('xl');
    expect(browseView.loupeAutoAdvance).toBe(true);
    expect(browseView.loupeInfoOpen).toBe(true);
    expect(browseView.tileInfo).toBe('full');
  });

  it('ignores an unknown grid size', async () => {
    stubStorage({
      gridSizeChoice: 'huge',
      loupeAutoAdvance: 'yes',
      loupeInfoOpen: 1,
      tileInfo: 'all'
    });

    const { browseView } = await import('./browseView.svelte');

    expect(browseView.gridSize).toBe('md');
    expect(browseView.loupeAutoAdvance).toBe(false);
    expect(browseView.loupeInfoOpen).toBe(false);
    expect(browseView.tileInfo).toBe('icons');
  });

  it('reads the old badges mode as icons', async () => {
    stubStorage({ tileInfo: 'badges' });
    const { browseView } = await import('./browseView.svelte');

    expect(browseView.tileInfo).toBe('icons');
  });

  it('writes every field whenever one changes', async () => {
    const values = stubStorage();

    const { browseView } = await import('./browseView.svelte');
    browseView.setLoupeAutoAdvance(true);
    browseView.setGridSize('lg');
    browseView.toggleLoupeInfo();
    browseView.cycleTileInfo();
    browseView.setFiltersOpen(true);
    browseView.setFilterSection('library', true);
    browseView.setFilterSection('place', true);
    browseView.setFilterSection('date', true);
    browseView.setFilterSection('library', false);

    expect(JSON.parse(values.get(storageKey) ?? '')).toEqual({
      filtersOpen: true,
      filterSections: ['place', 'date'],
      gridSizeChoice: 'lg',
      loupeAutoAdvance: true,
      loupeInfoOpen: true,
      tileInfo: 'full'
    });
  });

  it('restores the filter panel and drops unknown sections', async () => {
    stubStorage({ filtersOpen: true, filterSections: ['view', 'camera', 'lens', 'date'] });

    const { browseView } = await import('./browseView.svelte');

    expect(browseView.filtersOpen).toBe(true);
    expect(browseView.filterSections).toEqual(['camera', 'date']);
  });

  it('keeps the compact filter sheet transient and the docked panel persisted', async () => {
    const values = stubStorage({ filtersOpen: true });
    const { browseView } = await import('./browseView.svelte');

    browseView.setLayout({ mode: 'touch', compact: true });
    expect(browseView.filtersOpen).toBe(false);

    browseView.setFiltersOpen(true);
    expect(browseView.filtersOpen).toBe(true);
    expect(values.get(storageKey)).toBe(JSON.stringify({ filtersOpen: true }));

    browseView.setFiltersOpen(false);
    browseView.setLayout({ mode: 'touch', compact: false });
    expect(browseView.filtersOpen).toBe(true);

    browseView.setLayout({ mode: 'touch', compact: true });
    browseView.setFiltersOpen(true);
    browseView.setLayout({ mode: 'touch', compact: false });
    browseView.setLayout({ mode: 'touch', compact: true });
    expect(browseView.filtersOpen).toBe(false);

    browseView.setLayout({ mode: 'touch', compact: false });
    browseView.setFiltersOpen(false);
    expect(JSON.parse(values.get(storageKey) ?? '')).toMatchObject({ filtersOpen: false });
  });

  it('opens view and filters one at a time and never persists the view', async () => {
    const values = stubStorage({ filtersOpen: true });
    onLayout('touch', true);
    const { browseView } = await import('./browseView.svelte');

    browseView.setViewOpen(true);
    expect(browseView.viewOpen).toBe(true);
    expect(browseView.filtersOpen).toBe(false);
    expect(JSON.parse(values.get(storageKey) ?? '')).toMatchObject({ filtersOpen: false });

    browseView.filterPage = 'camera';
    browseView.setFiltersOpen(true);
    expect(browseView.viewOpen).toBe(false);
    expect(browseView.filtersOpen).toBe(true);
    expect(browseView.filterPage).toBe('camera');

    browseView.setViewOpen(true);
    expect(browseView.filterPage).toBeNull();
    expect(Object.keys(JSON.parse(values.get(storageKey) ?? ''))).not.toContain('viewOpen');

    onLayout('desktop');
    expect(browseView.viewOpen).toBe(false);
  });

  it('drops the filter page when the layout mode or compact changes', async () => {
    stubStorage();
    const { browseView } = await import('./browseView.svelte');
    browseView.setLayout({ mode: 'touch', compact: false });
    browseView.filterPage = 'people';

    browseView.setLayout({ mode: 'touch', compact: false });
    expect(browseView.filterPage).toBe('people');

    browseView.setLayout({ mode: 'desktop', compact: false });
    expect(browseView.filterPage).toBeNull();

    browseView.setLayout({ mode: 'touch', compact: false });
    browseView.filterPage = 'tags';
    browseView.setLayout({ mode: 'touch', compact: true });
    expect(browseView.filterPage).toBeNull();
  });

  it('drops the filter page when navigation leaves the page', async () => {
    stubStorage();
    const { browseView } = await import('./browseView.svelte');
    browseView.filterPage = 'people';

    browseView.navigated(new URL('http://x/photos'), new URL('http://x/photos?person=a'));
    expect(browseView.filterPage).toBe('people');

    browseView.navigated(new URL('http://x/photos'), new URL('http://x/folders'));
    expect(browseView.filterPage).toBeNull();
  });

  it('keeps the compact view sheet transient', async () => {
    stubStorage();
    const { browseView } = await import('./browseView.svelte');

    browseView.setLayout({ mode: 'touch', compact: true });
    browseView.setViewOpen(true);
    expect(browseView.viewOpen).toBe(true);

    browseView.setLayout({ mode: 'touch', compact: false });
    browseView.setLayout({ mode: 'touch', compact: true });
    expect(browseView.viewOpen).toBe(false);
  });

  it('keeps the compact info sheet transient and the docked info panel persisted', async () => {
    const values = stubStorage({ loupeInfoOpen: true });
    const { browseView } = await import('./browseView.svelte');

    browseView.setLayout({ mode: 'touch', compact: true });
    expect(browseView.loupeInfoOpen).toBe(false);

    browseView.toggleLoupeInfo();
    expect(browseView.loupeInfoOpen).toBe(true);
    expect(values.get(storageKey)).toBe(JSON.stringify({ loupeInfoOpen: true }));

    browseView.openLoupe('a');
    browseView.closeLoupe();
    expect(browseView.loupeInfoOpen).toBe(false);

    browseView.toggleLoupeInfo();
    browseView.setLayout({ mode: 'touch', compact: false });
    browseView.setLayout({ mode: 'touch', compact: true });
    expect(browseView.loupeInfoOpen).toBe(false);

    browseView.setLayout({ mode: 'touch', compact: false });
    expect(browseView.loupeInfoOpen).toBe(true);
    browseView.setLoupeInfoOpen(false);
    expect(JSON.parse(values.get(storageKey) ?? '')).toMatchObject({ loupeInfoOpen: false });
  });

  it('keeps one loupe filmstrip choice for compact touch and one for the other layouts', async () => {
    stubStorage();
    const { browseView } = await import('./browseView.svelte');
    const { ui } = await import('#lib/stores/ui.svelte.ts');

    expect(browseView.loupeFilmstripShown).toBe(true);
    browseView.setLayout({ mode: 'touch', compact: true });
    expect(browseView.loupeFilmstripShown).toBe(false);

    browseView.toggleLoupeFilmstrip();
    expect(browseView.loupeFilmstripShown).toBe(true);
    expect(ui.loupeFilmstripCollapsed).toBe(false);

    browseView.setLayout({ mode: 'touch', compact: false });
    browseView.toggleLoupeFilmstrip();
    expect(browseView.loupeFilmstripShown).toBe(false);
    browseView.setLayout({ mode: 'touch', compact: true });
    expect(browseView.loupeFilmstripShown).toBe(true);
  });

  it('cycles the tile info through none, icons and full', async () => {
    stubStorage({ tileInfo: 'full' });
    const { browseView } = await import('./browseView.svelte');

    browseView.cycleTileInfo();
    expect(browseView.tileInfo).toBe('none');
    browseView.cycleTileInfo();
    expect(browseView.tileInfo).toBe('icons');
  });
});

describe('grid size per layout', () => {
  it.each([
    ['desktop', false, false, 'md', 150],
    ['touch', false, false, 'lg', 190],
    ['touch', false, true, 'lg', 140],
    ['touch', true, false, 'lg', 190]
  ] as const)(
    'defaults on %s (wide %s, phone %s) to %s at %i px',
    async (mode, wide, phone, size, height) => {
      stubStorage();
      onLayout(mode, wide, phone);
      const { browseView } = await import('./browseView.svelte');

      expect(browseView.gridSize).toBe(size);
      expect(browseView.rowHeight).toBe(height);
    }
  );

  it('follows the layout until the user chooses and leaves the default unstored', async () => {
    const values = stubStorage();
    onLayout('touch');
    const { browseView } = await import('./browseView.svelte');

    browseView.setTileInfo('full');
    expect(JSON.parse(values.get(storageKey) ?? '')).not.toHaveProperty('gridSizeChoice');
    onLayout('desktop');
    expect(browseView.gridSize).toBe('md');

    browseView.setGridSize('sm');
    onLayout('touch', true);
    expect(browseView.gridSize).toBe('sm');
    expect(browseView.rowHeight).toBe(90);
    expect(JSON.parse(values.get(storageKey) ?? '')).toMatchObject({ gridSizeChoice: 'sm' });
  });

  it('ignores a legacy stored size and drops it on the next write', async () => {
    const values = stubStorage({ gridSize: 'md' });
    onLayout('touch', true);
    const { browseView } = await import('./browseView.svelte');

    expect(browseView.gridSize).toBe('lg');
    expect(browseView.rowHeight).toBe(190);

    browseView.setTileInfo('full');
    expect(JSON.parse(values.get(storageKey) ?? '')).not.toHaveProperty('gridSize');
  });

  it('sizes a stored choice per layout', async () => {
    stubStorage({ gridSizeChoice: 'md' });
    onLayout('touch', true);
    const { browseView } = await import('./browseView.svelte');

    expect(browseView.gridSize).toBe('md');
    expect(browseView.rowHeight).toBe(140);
  });
});
