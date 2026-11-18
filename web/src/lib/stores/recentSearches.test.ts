import { afterEach, describe, expect, it, vi } from 'vitest';

const KEY = 'immich-edit:recentSearches';

function stubStorage(initial?: unknown): Map<string, string> {
  const values = new Map<string, string>();
  if (initial !== undefined) values.set(KEY, JSON.stringify(initial));
  vi.stubGlobal('localStorage', {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: vi.fn((key: string, value: string) => values.set(key, value))
  });
  return values;
}

async function load() {
  return (await import('#lib/stores/recentSearches.svelte.ts')).recentSearches;
}

afterEach(() => {
  vi.unstubAllGlobals();
  vi.resetModules();
});

describe('recentSearches', () => {
  it('restores valid entries from this device', async () => {
    stubStorage({ items: ['boats', 7, 'harbour'] });
    expect((await load()).items).toEqual(['boats', 'harbour']);
  });

  it('records, removes and clears, persisting each change', async () => {
    const values = stubStorage();
    const recents = await load();

    recents.record('boats');
    recents.record('IMG_4120');
    expect(recents.items).toEqual(['IMG_4120', 'boats']);
    expect(JSON.parse(values.get(KEY) ?? '{}')).toEqual({ items: ['IMG_4120', 'boats'] });

    recents.remove('boats');
    expect(recents.items).toEqual(['IMG_4120']);

    recents.clear();
    expect(recents.items).toEqual([]);
    expect(JSON.parse(values.get(KEY) ?? '{}')).toEqual({ items: [] });
  });

  it('merges a growing query only within one typing session', async () => {
    stubStorage();
    const recents = await load();
    const typing = Symbol('typing');

    recents.record('red', typing);
    recents.record('red car', typing);
    expect(recents.items).toEqual(['red car']);

    const later = Symbol('later');
    recents.record('red', later);
    recents.record('red car', Symbol('again'));
    expect(recents.items).toEqual(['red car', 'red']);

    recents.record('red', later);
    recents.record('red boat');
    expect(recents.items).toEqual(['red boat', 'red', 'red car']);
  });

  it('skips the storage write when nothing changed', async () => {
    stubStorage({ items: ['boats'] });
    const recents = await load();

    recents.record('boats');
    recents.remove('harbour');
    expect(localStorage.setItem).not.toHaveBeenCalled();
  });
});
