import { describe, expect, it } from 'vitest';
import { mergeWindow, singleAnchor, windowEdge, windowEdges } from './feedWindow';

const master = (id: string) => ({ id });
const copy = (id: string, of: string) => ({ id, copyOf: of });

describe('windowEdge', () => {
  it('anchors on the master at each end', () => {
    const list = [master('a'), master('b'), copy('b_1', 'b')];
    expect(windowEdge(list, 'first')).toBe('a');
    expect(windowEdge(list, 'last')).toBe('b');
  });

  it('has no edge for an empty list', () => {
    expect(windowEdge([], 'first')).toBeNull();
  });
});

describe('windowEdges', () => {
  it('walks inward over distinct masters from either end', () => {
    const list = [master('a'), copy('a_1', 'a'), master('b'), master('c'), copy('c_1', 'c')];
    expect(windowEdges(list, 'first', 3)).toEqual(['a', 'b', 'c']);
    expect(windowEdges(list, 'last', 2)).toEqual(['c', 'b']);
  });
});

describe('singleAnchor', () => {
  it('is true only for one master and its copies', () => {
    expect(singleAnchor([master('a'), copy('a_1', 'a')])).toBe(true);
    expect(singleAnchor([master('a'), master('b')])).toBe(false);
    expect(singleAnchor([])).toBe(false);
  });
});

describe('mergeWindow', () => {
  it('prepends only the photos that are not loaded yet', () => {
    const loaded = [master('c'), copy('c_1', 'c'), master('d')];
    const incoming = [master('a'), master('b'), master('c'), copy('c_1', 'c')];
    expect(mergeWindow(loaded, incoming, 'before').map((asset) => asset.id)).toEqual([
      'a',
      'b',
      'c',
      'c_1',
      'd'
    ]);
  });

  it('appends after the loaded end without duplicates', () => {
    const loaded = [master('a'), master('b')];
    const incoming = [master('b'), master('c'), master('c')];
    expect(mergeWindow(loaded, incoming, 'after').map((asset) => asset.id)).toEqual([
      'a',
      'b',
      'c'
    ]);
  });
});
