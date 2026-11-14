import { describe, expect, it } from 'vitest';
import { countFailures, runPool } from './pool';

function deferred(): { promise: Promise<void>; resolve: () => void } {
  let resolve: () => void = () => {};
  const promise = new Promise<void>((r) => (resolve = r));
  return { promise, resolve };
}

describe('runPool', () => {
  it('runs every item once', async () => {
    const seen: number[] = [];
    await runPool([1, 2, 3, 4, 5], 2, async (n) => {
      seen.push(n);
    });
    expect(seen.sort()).toEqual([1, 2, 3, 4, 5]);
  });

  it('never runs more than the limit at once', async () => {
    const gates = [deferred(), deferred(), deferred()];
    let running = 0;
    let peak = 0;
    const done = runPool(gates, 2, async (gate) => {
      running += 1;
      peak = Math.max(peak, running);
      await gate.promise;
      running -= 1;
    });
    await Promise.resolve();
    expect(running).toBe(2);
    gates.forEach((gate) => gate.resolve());
    await done;
    expect(peak).toBe(2);
  });

  it('keeps undefined items', async () => {
    const seen: (string | undefined)[] = [];
    await runPool(['a', undefined], 4, async (item) => {
      seen.push(item);
    });
    expect(seen).toEqual(['a', undefined]);
  });

  it('finishes at once with nothing to do', async () => {
    await expect(runPool([], 3, () => Promise.reject(new Error('ran')))).resolves.toBeUndefined();
  });
});

describe('countFailures', () => {
  it('counts false results and thrown errors', async () => {
    const failed = await countFailures([1, 2, 3, 4], 2, async (n) => {
      if (n === 4) throw new Error('boom');
      return n !== 2;
    });
    expect(failed).toBe(2);
  });
});
