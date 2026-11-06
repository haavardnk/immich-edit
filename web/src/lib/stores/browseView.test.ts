import { afterEach, describe, expect, it, vi } from 'vitest';
import type { LayoutMode } from '#lib/device/layoutRules.ts';

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
    expect(browseView.tileInfo).toBe('badges');
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
    browseView.setFilterSection('view', true);
    browseView.setFilterSection('library', false);

    expect(JSON.parse(values.get(storageKey) ?? '')).toEqual({
      filtersOpen: true,
      filterSections: ['place', 'view'],
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
    expect(browseView.filterSections).toEqual(['view', 'camera', 'date']);
  });

  it('keeps the compact filter sheet transient and the docked panel persisted', async () => {
    const values = stubStorage({ filtersOpen: true });
    const { browseView } = await import('./browseView.svelte');

    browseView.setCompact(true);
    expect(browseView.filtersOpen).toBe(false);

    browseView.setFiltersOpen(true);
    expect(browseView.filtersOpen).toBe(true);
    expect(values.get(storageKey)).toBe(JSON.stringify({ filtersOpen: true }));

    browseView.setFiltersOpen(false);
    browseView.setCompact(false);
    expect(browseView.filtersOpen).toBe(true);

    browseView.setCompact(true);
    browseView.setFiltersOpen(true);
    browseView.setCompact(false);
    browseView.setCompact(true);
    expect(browseView.filtersOpen).toBe(false);

    browseView.setCompact(false);
    browseView.setFiltersOpen(false);
    expect(JSON.parse(values.get(storageKey) ?? '')).toMatchObject({ filtersOpen: false });
  });

  it('keeps the compact info sheet transient and the docked info panel persisted', async () => {
    const values = stubStorage({ loupeInfoOpen: true });
    const { browseView } = await import('./browseView.svelte');

    browseView.setCompact(true);
    expect(browseView.loupeInfoOpen).toBe(false);

    browseView.toggleLoupeInfo();
    expect(browseView.loupeInfoOpen).toBe(true);
    expect(values.get(storageKey)).toBe(JSON.stringify({ loupeInfoOpen: true }));

    browseView.openLoupe('a');
    browseView.closeLoupe();
    expect(browseView.loupeInfoOpen).toBe(false);

    browseView.toggleLoupeInfo();
    browseView.setCompact(false);
    browseView.setCompact(true);
    expect(browseView.loupeInfoOpen).toBe(false);

    browseView.setCompact(false);
    expect(browseView.loupeInfoOpen).toBe(true);
    browseView.setLoupeInfoOpen(false);
    expect(JSON.parse(values.get(storageKey) ?? '')).toMatchObject({ loupeInfoOpen: false });
  });

  it('keeps one loupe filmstrip choice for compact touch and one for the other layouts', async () => {
    stubStorage();
    const { browseView } = await import('./browseView.svelte');
    const { ui } = await import('#lib/stores/ui.svelte.ts');

    expect(browseView.loupeFilmstripShown).toBe(true);
    browseView.setCompact(true);
    expect(browseView.loupeFilmstripShown).toBe(false);

    browseView.toggleLoupeFilmstrip();
    expect(browseView.loupeFilmstripShown).toBe(true);
    expect(ui.loupeFilmstripCollapsed).toBe(false);

    browseView.setCompact(false);
    browseView.toggleLoupeFilmstrip();
    expect(browseView.loupeFilmstripShown).toBe(false);
    browseView.setCompact(true);
    expect(browseView.loupeFilmstripShown).toBe(true);
  });

  it('cycles the tile info through none, badges and full', async () => {
    stubStorage({ tileInfo: 'full' });
    const { browseView } = await import('./browseView.svelte');

    browseView.cycleTileInfo();
    expect(browseView.tileInfo).toBe('none');
    browseView.cycleTileInfo();
    expect(browseView.tileInfo).toBe('badges');
  });
});

describe('grid size per layout', () => {
  it.each([
    ['desktop', false, false, 'md', 150],
    ['touch', false, false, 'lg', 112],
    ['touch', false, true, 'lg', 140],
    ['touch', true, false, 'xl', 160]
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
    onLayout('touch', true);
    expect(browseView.gridSize).toBe('xl');

    browseView.setGridSize('sm');
    onLayout('desktop');
    expect(browseView.gridSize).toBe('sm');
    expect(browseView.rowHeight).toBe(110);
    expect(JSON.parse(values.get(storageKey) ?? '')).toMatchObject({ gridSizeChoice: 'sm' });
  });

  it('ignores a legacy stored size and drops it on the next write', async () => {
    const values = stubStorage({ gridSize: 'md' });
    onLayout('touch', true);
    const { browseView } = await import('./browseView.svelte');

    expect(browseView.gridSize).toBe('xl');
    expect(browseView.rowHeight).toBe(160);

    browseView.setTileInfo('full');
    expect(JSON.parse(values.get(storageKey) ?? '')).not.toHaveProperty('gridSize');
  });

  it('sizes a stored choice per layout', async () => {
    stubStorage({ gridSizeChoice: 'md' });
    onLayout('touch', true);
    const { browseView } = await import('./browseView.svelte');

    expect(browseView.gridSize).toBe('md');
    expect(browseView.rowHeight).toBe(80);
  });
});
