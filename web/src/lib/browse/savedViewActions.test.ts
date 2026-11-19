import { isCurrentView, openSavedView } from '#lib/browse/savedViewActions.ts';
import { FILTER_DEFAULTS, browseControls } from '#lib/stores/browseControls.svelte.ts';
import { browseView } from '#lib/stores/browseView.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { goto } from '$app/navigation';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const state = vi.hoisted(() => ({
  url: new URL('http://localhost/photos?favorite=1&country=Norway')
}));

vi.mock('$app/state', () => ({
  page: {
    get url() {
      return state.url;
    },
    params: {}
  }
}));

vi.mock('$app/navigation', () => ({ goto: vi.fn() }));

const NEWEST = { field: 'taken', direction: 'desc' } as const;

beforeEach(() => {
  state.url = new URL('http://localhost/photos?favorite=1&country=Norway');
  browseControls.enter('timeline', FILTER_DEFAULTS);
  browseControls.applySort(NEWEST, 'timeline');
  browseView.setGridSize('md');
  vi.mocked(goto).mockResolvedValue(undefined);
});

afterEach(() => {
  browseControls.settleQueuedSort(false);
  vi.mocked(goto).mockReset();
  vi.restoreAllMocks();
});

describe('isCurrentView', () => {
  it('ignores the order of filter params, sort and size', () => {
    expect(isCurrentView('/photos?country=Norway&favorite=1&sort=added-asc&size=xl')).toBe(true);
    expect(isCurrentView('/photos?country=Norway')).toBe(false);
    expect(isCurrentView('/.//evil.com')).toBe(false);
  });
});

describe('openSavedView', () => {
  it('applies sort and size in place when the view is already open', async () => {
    await openSavedView('/photos?country=Norway&favorite=1&sort=taken-asc&size=xl');
    expect(goto).not.toHaveBeenCalled();
    expect(browseView.gridSize).toBe('xl');
    expect(browseControls.sort).toEqual({ field: 'taken', direction: 'asc' });
  });

  it('keeps the old page sorted as it was until the target page takes the sort', async () => {
    let sortDuringNavigation = browseControls.sort;
    vi.mocked(goto).mockImplementation(async () => {
      sortDuringNavigation = browseControls.sort;
      browseControls.applyFilters({ ...FILTER_DEFAULTS, rating: 3 });
    });
    await openSavedView('/photos?rating=3&sort=taken-asc&size=lg');
    expect(goto).toHaveBeenCalledWith('/photos?rating=3');
    expect(sortDuringNavigation).toEqual(NEWEST);
    expect(browseControls.sort).toEqual({ field: 'taken', direction: 'asc' });
    expect(browseView.gridSize).toBe('lg');
  });

  it('applies the sort after navigating when no route took it', async () => {
    await openSavedView('/photos?rating=3&sort=taken-asc');
    expect(browseControls.sort).toEqual({ field: 'taken', direction: 'asc' });
  });

  it('reports a failed navigation and leaves the sort alone', async () => {
    const fail = vi.spyOn(toasts, 'fail');
    vi.mocked(goto).mockRejectedValue(new Error('offline'));
    await expect(openSavedView('/photos?rating=3&sort=taken-asc')).resolves.toBeUndefined();
    expect(fail).toHaveBeenCalledWith('Failed to open view', expect.any(Error));
    browseControls.applyFilters(FILTER_DEFAULTS);
    expect(browseControls.sort).toEqual(NEWEST);
  });

  it('refuses a view whose address leaves the site', async () => {
    const push = vi.spyOn(toasts, 'push');
    await openSavedView('/a/..//evil.com?size=xl');
    expect(goto).not.toHaveBeenCalled();
    expect(browseView.gridSize).toBe('md');
    expect(push).toHaveBeenCalledWith('error', 'This saved view has an invalid address');
  });
});
