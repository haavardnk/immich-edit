import { describe, expect, it } from 'vitest';
import { putBounded } from './boundedRecord';

describe('putBounded', () => {
  it('evicts oldest keys beyond max', () => {
    let cache: Record<string, number> = {};
    let order: string[] = [];
    for (const k of ['a', 'b', 'c']) {
      const next = putBounded(cache, order, k, k.charCodeAt(0), 2);
      cache = next.record;
      order = next.order;
    }
    expect(cache).toEqual({ b: 98, c: 99 });
    expect(order).toEqual(['b', 'c']);
  });

  it('refreshes recency on re-insert without growing', () => {
    let cache: Record<string, number> = {};
    let order: string[] = [];
    for (const [k, v] of [
      ['a', 1],
      ['b', 2],
      ['a', 3],
      ['c', 4]
    ] as const) {
      const next = putBounded(cache, order, k, v, 2);
      cache = next.record;
      order = next.order;
    }
    expect(cache).toEqual({ a: 3, c: 4 });
    expect(order).toEqual(['a', 'c']);
  });
});
