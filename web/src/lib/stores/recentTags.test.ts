import { afterEach, describe, expect, it, vi } from 'vitest';

const KEY = 'immich-edit:recentTags';

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
  return (await import('#lib/stores/recentTags.svelte.ts')).recentTags;
}

function stored(values: Map<string, string>): string[] {
  const parsed = JSON.parse(values.get(KEY) ?? '{}') as { tags?: { id: string }[] };
  return (parsed.tags ?? []).map((tag) => tag.id);
}

afterEach(() => {
  vi.unstubAllGlobals();
  vi.resetModules();
});

describe('recentTags', () => {
  it('restores the recent tags of this browser', async () => {
    stubStorage({ tags: [{ id: 'a', name: 'Sky', value: 'Sky' }, { id: 1 }] });
    expect((await load()).tags.map((tag) => tag.id)).toEqual(['a']);
  });

  it('records the newest first and persists it', async () => {
    const values = stubStorage();
    const recents = await load();
    recents.record({ id: 'a', name: 'Sky', value: 'Sky' });
    recents.record({ id: 'b', name: 'Sea', value: 'Sea' });
    expect(recents.tags.map((tag) => tag.id)).toEqual(['b', 'a']);
    expect(stored(values)).toEqual(['b', 'a']);
  });

  it('forgets tags the library no longer lists', async () => {
    const values = stubStorage({
      tags: [
        { id: 'a', name: 'Sky', value: 'Sky' },
        { id: 'b', name: 'Sea', value: 'Sea' }
      ]
    });
    const recents = await load();
    recents.keep([{ id: 'b', name: 'Sea', value: 'Sea', createdAt: null }]);
    expect(recents.tags.map((tag) => tag.id)).toEqual(['b']);
    expect(stored(values)).toEqual(['b']);
  });

  it('clears on request', async () => {
    const values = stubStorage({ tags: [{ id: 'a', name: 'Sky', value: 'Sky' }] });
    const recents = await load();
    recents.clear();
    expect(recents.tags).toEqual([]);
    expect(stored(values)).toEqual([]);
  });
});
