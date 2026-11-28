import type { TagRef } from '#lib/types/asset.ts';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const A = '0f8fad5b-d9cb-469f-a165-70867728950e';
const B = '7c9e6679-7425-40de-944b-e07fc1f90ae7';

const trip: TagRef = { id: 't1', name: 'Trip', value: 'Trip' };
const family: TagRef = { id: 't2', name: 'Family', value: 'Family' };

beforeEach(() => {
  vi.resetModules();
});

afterEach(() => {
  vi.useRealTimers();
});

async function store() {
  return (await import('./tagCache.svelte')).tagCache;
}

async function retryMs() {
  return (await import('./tagCache.svelte')).RETRY_MS;
}

function answering(found: Record<string, TagRef[]>) {
  return vi.fn((ids: readonly string[]) =>
    Promise.resolve(Object.fromEntries(ids.flatMap((id) => (found[id] ? [[id, found[id]]] : []))))
  );
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

  it('leaves photos missing from a batch unloaded and waits before asking again', async () => {
    vi.useFakeTimers({ toFake: ['Date'] });
    const tags = await store();
    const fetch = answering({ [A]: [trip] });
    await tags.loadMany([A, B], fetch);
    expect(tags.get(A)).toEqual([trip]);
    expect(tags.get(B)).toBeUndefined();
    vi.setSystemTime(Date.now() + (await retryMs()) - 1);
    await tags.loadMany([B], fetch);
    expect(fetch).toHaveBeenCalledTimes(1);
    vi.setSystemTime(Date.now() + 1);
    await tags.loadMany([B], fetch);
    expect(fetch).toHaveBeenLastCalledWith([B]);
  });

  it('waits after a failed batch until a clear', async () => {
    const tags = await store();
    const offline = vi.fn(() => Promise.reject(new Error('offline')));
    await tags.loadMany([A, B], offline);
    await tags.loadMany([A, B], offline);
    expect(offline).toHaveBeenCalledTimes(1);
    tags.clear();
    const fetch = answering({ [A]: [family], [B]: [] });
    await tags.loadMany([A, B], fetch);
    expect(fetch).toHaveBeenCalledWith([A, B]);
    expect(tags.get(B)).toEqual([]);
  });

  it('does not hold back a photo whose failed batch a clear overtook', async () => {
    const tags = await store();
    let fail: (error: Error) => void = () => {};
    const running = tags.loadMany([A], () => new Promise((_, reject) => (fail = reject)));
    tags.clear();
    fail(new Error('offline'));
    await running;
    await tags.loadMany([A], answering({ [A]: [trip] }));
    expect(tags.get(A)).toEqual([trip]);
  });

  it('loads a photo alone when the batch it joined left it out', async () => {
    const tags = await store();
    let answer: (found: Record<string, TagRef[]>) => void = () => {};
    const batch = tags.loadMany([A], () => new Promise((done) => (answer = done)));
    const fetch = vi.fn(() => Promise.resolve([trip]));
    const single = tags.load(A, fetch);
    answer({});
    await Promise.all([batch, single]);
    expect(fetch).toHaveBeenCalledTimes(1);
    expect(tags.get(A)).toEqual([trip]);
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
