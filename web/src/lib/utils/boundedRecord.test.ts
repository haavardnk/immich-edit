import { describe, expect, it } from 'vitest';
import { putAllBounded, putBounded } from './boundedRecord';

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

describe('putAllBounded', () => {
  it('stores a batch in one copy, refreshing and evicting the oldest keys', () => {
    const record = { a: 1, b: 2, c: 3 };
    const next = putAllBounded(
      record,
      ['a', 'b', 'c'],
      [
        ['a', 10],
        ['d', 4]
      ],
      3
    );
    expect(next.record).toEqual({ c: 3, a: 10, d: 4 });
    expect(next.order).toEqual(['c', 'a', 'd']);
    expect(record).toEqual({ a: 1, b: 2, c: 3 });
  });
});
