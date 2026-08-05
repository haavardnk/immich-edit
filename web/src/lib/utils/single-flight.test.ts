import { describe, it, expect, vi } from 'vitest';
import { SingleFlight } from './single-flight';

function deferred<T>() {
  let resolve!: (v: T) => void;
  let reject!: (e: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

describe('SingleFlight', () => {
  it('runs a single submission and reports the result', async () => {
    const results: Array<[number, number]> = [];
    const sf = new SingleFlight<number, number>(
      async (n) => n * 2,
      (args, result) => results.push([args, result])
    );
    sf.submit(5);
    await Promise.resolve();
    await Promise.resolve();
    expect(results).toEqual([[5, 10]]);
  });

  it('queues last-wins while running and aborts the in-flight call', async () => {
    const d = deferred<number>();
    const seen: number[] = [];
    const aborted: boolean[] = [];
    const sf = new SingleFlight<number, number>(
      async (n, signal) => {
        seen.push(n);
        if (n === 1) {
          await d.promise;
          aborted.push(signal.aborted);
          return n;
        }
        return n;
      },
      () => {}
    );
    sf.submit(1);
    await Promise.resolve();
    sf.submit(2);
    sf.submit(3);
    d.resolve(1);
    await Promise.resolve();
    await Promise.resolve();
    await Promise.resolve();
    expect(seen[0]).toBe(1);
    expect(seen[seen.length - 1]).toBe(3);
    expect(aborted[0]).toBe(true);
  });

  it('swallows AbortError but forwards other errors', async () => {
    const onError = vi.fn();
    const sf = new SingleFlight<string, void>(
      async (kind) => {
        if (kind === 'abort') throw new DOMException('aborted', 'AbortError');
        throw new Error('boom');
      },
      () => {},
      onError
    );
    sf.submit('abort');
    await Promise.resolve();
    await Promise.resolve();
    expect(onError).not.toHaveBeenCalled();

    sf.submit('fail');
    await Promise.resolve();
    await Promise.resolve();
    expect(onError).toHaveBeenCalledTimes(1);
    expect((onError.mock.calls[0][0] as Error).message).toBe('boom');
  });

  it('cancel aborts the current call and clears the queue', async () => {
    const d = deferred<number>();
    const onResult = vi.fn();
    const sf = new SingleFlight<number, number>(async (_n, signal) => {
      await d.promise;
      if (signal.aborted) throw new DOMException('aborted', 'AbortError');
      return 1;
    }, onResult);
    sf.submit(1);
    await Promise.resolve();
    sf.cancel();
    d.resolve(1);
    await Promise.resolve();
    await Promise.resolve();
    expect(onResult).not.toHaveBeenCalled();
  });
});
