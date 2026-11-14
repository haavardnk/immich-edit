import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({ findEarlierExports: vi.fn() }));

vi.mock('#lib/api/exportOutputs.ts', () => ({ findEarlierExports: mocks.findEarlierExports }));

import type { EarlierExports as Found } from '#lib/api/exportOutputs.ts';
import { EarlierExports } from './earlierExports.svelte';

const FOUND: Found = {
  photos: 1,
  files: [{ filename: 'a_edit.jpg', exported_at: '2026-11-15T11:00:00Z' }]
};

const NONE: Found = { photos: 0, files: [] };

function deferred<T>(): { promise: Promise<T>; resolve: (value: T) => void } {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((r) => (resolve = r));
  return { promise, resolve };
}

beforeEach(() => {
  mocks.findEarlierExports.mockReset();
});

describe('EarlierExports', () => {
  it('shows a run only when an earlier export exists', async () => {
    mocks.findEarlierExports.mockResolvedValueOnce(NONE).mockResolvedValueOnce(FOUND);
    const earlier = new EarlierExports();

    earlier.watch({ assetIds: ['a'], presets: [] });
    await vi.waitFor(() => expect(mocks.findEarlierExports).toHaveBeenCalledTimes(1));
    await Promise.resolve();
    expect(earlier.run).toBeNull();

    earlier.refresh();
    await vi.waitFor(() => expect(earlier.run).not.toBeNull());
    expect(earlier.run).toEqual({ found: FOUND, total: 1, presets: 0 });
    expect(mocks.findEarlierExports).toHaveBeenLastCalledWith(['a'], []);
  });

  it('hands out the choice once and falls back to Keep both', async () => {
    mocks.findEarlierExports.mockResolvedValue(FOUND);
    const earlier = new EarlierExports();
    earlier.watch({ assetIds: ['a'], presets: ['Web'] });
    await vi.waitFor(() => expect(earlier.run).not.toBeNull());

    earlier.policy = 'replace';
    expect(earlier.take()).toBe('replace');
    expect(earlier.policy).toBe('keep_both');
    expect(earlier.take()).toBe('keep_both');
  });

  it('keeps both when the card is hidden', () => {
    const earlier = new EarlierExports();
    earlier.policy = 'replace';
    expect(earlier.take()).toBe('keep_both');
  });

  it('resets the choice and reloads only when the query changes', async () => {
    mocks.findEarlierExports.mockResolvedValue(FOUND);
    const earlier = new EarlierExports();
    earlier.watch({ assetIds: ['a', 'b'], presets: [] });
    await vi.waitFor(() => expect(earlier.run).not.toBeNull());
    earlier.policy = 'skip';

    earlier.watch({ assetIds: ['a', 'b'], presets: [] });
    expect(earlier.policy).toBe('skip');
    expect(mocks.findEarlierExports).toHaveBeenCalledTimes(1);

    earlier.watch({ assetIds: ['a'], presets: [] });
    expect(earlier.policy).toBe('keep_both');
    expect(earlier.run).toBeNull();

    earlier.watch(null);
    await Promise.resolve();
    expect(earlier.run).toBeNull();
  });

  it('ignores an answer for an older query', async () => {
    const slow = deferred<Found>();
    mocks.findEarlierExports.mockReturnValueOnce(slow.promise).mockResolvedValueOnce(NONE);
    const earlier = new EarlierExports();
    earlier.watch({ assetIds: ['a'], presets: [] });
    earlier.watch({ assetIds: ['b'], presets: [] });
    slow.resolve(FOUND);
    await slow.promise;
    await Promise.resolve();
    expect(earlier.run).toBeNull();
  });

  it('hides the card when the lookup fails', async () => {
    mocks.findEarlierExports.mockRejectedValue(new Error('offline'));
    const earlier = new EarlierExports();
    earlier.watch({ assetIds: ['a'], presets: [] });
    await vi.waitFor(() => expect(mocks.findEarlierExports).toHaveBeenCalled());
    await Promise.resolve();
    expect(earlier.run).toBeNull();
  });
});
