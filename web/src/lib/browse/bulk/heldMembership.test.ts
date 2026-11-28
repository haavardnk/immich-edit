import { describe, expect, it } from 'vitest';
import { MEMBERSHIP_CAP, heldCounts } from './heldMembership';

const held: Record<string, { id: string }[]> = {
  a: [{ id: 'trip' }, { id: 'family' }],
  b: [{ id: 'trip' }],
  c: []
};

describe('held membership', () => {
  it('counts how many photos hold each album or tag', () => {
    expect(heldCounts(['a', 'b', 'c'], (id) => held[id])).toEqual(
      new Map([
        ['trip', 2],
        ['family', 1]
      ])
    );
  });

  it('knows nothing while any photo is unloaded or failed', () => {
    expect(heldCounts(['a', 'missing'], (id) => held[id])).toBeNull();
    expect(heldCounts(['a'], () => null)).toBeNull();
  });

  it('knows nothing above the cap', () => {
    const sources = Array.from({ length: MEMBERSHIP_CAP + 1 }, () => 'c');
    expect(heldCounts(sources, (id) => held[id])).toBeNull();
    expect(heldCounts(sources.slice(1), (id) => held[id])).toEqual(new Map());
  });
});
