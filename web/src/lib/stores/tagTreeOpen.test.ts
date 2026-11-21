import { afterEach, describe, expect, it, vi } from 'vitest';

const storageKey = 'immich-edit:tagTree';

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

async function load(initial?: unknown) {
  const values = stubStorage(initial);
  const { tagTreeOpen } = await import('./tagTreeOpen.svelte');
  return { tagTreeOpen, saved: () => JSON.parse(values.get(storageKey) ?? 'null') as unknown };
}

describe('tag tree open branches', () => {
  it('restores string keys and drops anything else', async () => {
    const { tagTreeOpen } = await load({ open: ['travel', 3, 'norway'] });
    expect(tagTreeOpen.isOpen('travel')).toBe(true);
    expect(tagTreeOpen.isOpen('norway')).toBe(true);
    expect(tagTreeOpen.isOpen('3')).toBe(false);
  });

  it('persists a toggle both ways', async () => {
    const { tagTreeOpen, saved } = await load();
    tagTreeOpen.toggle('travel');
    expect(saved()).toEqual({ open: ['travel'] });
    tagTreeOpen.toggle('travel');
    expect(saved()).toEqual({ open: [] });
  });

  it('opens and closes a set of branches and reports when all are open', async () => {
    const { tagTreeOpen } = await load({ open: ['other'] });
    expect(tagTreeOpen.allOpen(['a', 'b'])).toBe(false);
    tagTreeOpen.openAll(['a', 'b']);
    expect(tagTreeOpen.allOpen(['a', 'b'])).toBe(true);
    tagTreeOpen.closeAll(['a', 'b']);
    expect(tagTreeOpen.isOpen('a')).toBe(false);
    expect(tagTreeOpen.isOpen('other')).toBe(true);
    expect(tagTreeOpen.allOpen([])).toBe(false);
  });
});
