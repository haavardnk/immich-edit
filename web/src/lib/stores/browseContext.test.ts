import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';

const KEY = 'immich-edit:browseContext';

const FILTERS = {
  favoriteOnly: true,
  rating: 3 as const,
  filename: 'IMG_0001',
  visibility: 'archive' as const,
  takenAfter: '2024-01-01',
  takenBefore: '2024-02-01',
  excludeRejected: true
};

let store: Record<string, string>;

async function load() {
  vi.resetModules();
  return import('./browseContext');
}

beforeEach(() => {
  store = {};
  vi.stubGlobal('localStorage', {
    getItem: (k: string) => store[k] ?? null,
    setItem: (k: string, v: string) => {
      store[k] = v;
    },
    removeItem: (k: string) => {
      delete store[k];
    }
  });
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('browseContext', () => {
  it('round-trips every filter for the matching path', async () => {
    const { rememberBrowseContext, recallBrowseFilters } = await load();
    rememberBrowseContext('/albums/a1', FILTERS);
    expect(recallBrowseFilters('/albums/a1')).toEqual(FILTERS);
  });

  it('ignores a snapshot taken on a different path', async () => {
    const { rememberBrowseContext, recallBrowseFilters } = await load();
    rememberBrowseContext('/photos', FILTERS);
    expect(recallBrowseFilters('/favorites')).toBeNull();
  });

  it.each([
    ['rating', { rating: 9 }, 'rating', 'any'],
    ['visibility', { visibility: 'everywhere' }, 'visibility', 'timeline'],
    ['filename', { filename: 42 }, 'filename', ''],
    ['favoriteOnly', { favoriteOnly: 'yes' }, 'favoriteOnly', false]
  ])('replaces a corrupt %s with its default', async (_name, patch, field, expected) => {
    const { recallBrowseFilters } = await load();
    store[KEY] = JSON.stringify({ path: '/photos', filters: { ...FILTERS, ...patch } });
    const filters = recallBrowseFilters('/photos');
    expect(filters).not.toBeNull();
    expect(filters?.[field as keyof typeof FILTERS]).toBe(expected);
  });
});
