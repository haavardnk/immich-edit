import type { TagRef } from '#lib/types/asset.ts';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const A = '0f8fad5b-d9cb-469f-a165-70867728950e';
const B = '7c9e6679-7425-40de-944b-e07fc1f90ae7';

const trip: TagRef = { id: 't1', name: 'Trip', value: 'Trip' };
const family: TagRef = { id: 't2', name: 'Family', value: 'Family' };

beforeEach(() => {
  vi.resetModules();
});

async function store() {
  return (await import('./tagCache.svelte')).tagCache;
}

describe('tag cache', () => {
  it('shares one entry between a photo and its copies', async () => {
    const tags = await store();
    tags.set(`${A}_2`, [trip]);
    expect(tags.get(A)).toEqual([trip]);
    tags.update(A, (current) => [...current, family]);
    expect(tags.get(`${A}_3`)).toEqual([trip, family]);
  });

  it('loads many photos in batches, skipping cached and loading ones', async () => {
    const tags = await store();
    tags.set(A, [trip]);
    const fetch = vi.fn((ids: readonly string[]) =>
      Promise.resolve(Object.fromEntries(ids.map((id) => [id, [family]])))
    );
    const ids = Array.from(
      { length: 120 },
      (_, i) => `${i.toString(16).padStart(8, '0')}-0000-4000-8000-000000000000`
    );
    await tags.loadMany([A, B, `${B}_1`, ...ids], fetch);
    expect(fetch.mock.calls.map(([batch]) => batch.length)).toEqual([100, 21]);
    expect(tags.get(A)).toEqual([trip]);
    expect(tags.get(B)).toEqual([family]);
  });

  it('applies a change made while the batch loads', async () => {
    const tags = await store();
    let answer: (found: Record<string, TagRef[]>) => void = () => {};
    const running = tags.loadMany([A], () => new Promise((done) => (answer = done)));
    tags.update(A, (current) => [...current, family]);
    answer({ [A]: [trip] });
    await running;
    expect(tags.get(A)).toEqual([trip, family]);
  });

  it('drops a batch that a clear overtook and swallows failed batches', async () => {
    const tags = await store();
    let answer: (found: Record<string, TagRef[]>) => void = () => {};
    const running = tags.loadMany([A], () => new Promise((done) => (answer = done)));
    tags.clear();
    answer({ [A]: [trip] });
    await running;
    expect(tags.get(A)).toBeUndefined();
    await tags.loadMany([B], () => Promise.reject(new Error('offline')));
    expect(tags.get(B)).toBeUndefined();
  });

  it('passes a single load failure to its caller', async () => {
    const tags = await store();
    await expect(tags.load(A, () => Promise.reject(new Error('offline')))).rejects.toThrow(
      'offline'
    );
    await tags.load(A, () => Promise.resolve([trip]));
    expect(tags.get(A)).toEqual([trip]);
  });
});
