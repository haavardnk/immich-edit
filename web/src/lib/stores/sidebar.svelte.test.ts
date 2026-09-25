import { afterEach, describe, expect, it, vi } from 'vitest';

const storageKey = 'immich-edit:sidebar';

afterEach(() => {
  vi.unstubAllGlobals();
  vi.resetModules();
});

function stubStorage(initial?: unknown): Map<string, string> {
  const values = new Map<string, string>();
  if (initial !== undefined) values.set(storageKey, JSON.stringify(initial));
  vi.stubGlobal('localStorage', {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => values.set(key, value)
  });
  return values;
}

describe('sidebar sections', () => {
  it.each([
    ['/albums/a1', 'albums'],
    ['/people/p1', 'people'],
    ['/tags/t1', 'tags'],
    ['/folders', 'folders'],
    ['/photos', null],
    ['/albums', null]
  ])('maps %s to %s', async (path, section) => {
    stubStorage();
    const { sectionForPath } = await import('./sidebar.svelte');
    expect(sectionForPath(path)).toBe(section);
  });

  it('restores known sections and drops unknown ones', async () => {
    stubStorage({ expanded: ['tags', 'bogus', 'people'] });
    const { sidebar } = await import('./sidebar.svelte');
    expect(sidebar.expanded).toEqual(['people', 'tags']);
  });

  it('persists toggles and reveals the section that owns a page', async () => {
    const values = stubStorage();
    const { sidebar } = await import('./sidebar.svelte');

    expect(sidebar.toggle('tags')).toBe(true);
    sidebar.reveal('/albums/a1');
    sidebar.reveal('/albums/a2');
    expect(sidebar.toggle('tags')).toBe(false);

    expect(JSON.parse(values.get(storageKey) ?? '')).toEqual({ expanded: ['albums'] });
  });
});
