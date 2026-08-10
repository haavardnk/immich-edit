import { describe, expect, it } from 'vitest';
import { hiresEdge } from './preview-size';

describe('hiresEdge', () => {
  it.each([
    [900, 1, 100, 1024],
    [900, 2, 100, 2048],
    [900, 3, 100, 2048],
    [900, 1, 200, 2048],
    [1400, 2, 100, 3072],
    [3000, 2, 400, 4096],
    [100, 0, 100, 1024]
  ])('viewport=%i dpr=%i zoom=%i -> %i', (viewport, dpr, zoom, expected) => {
    expect(hiresEdge(viewport, dpr, zoom)).toBe(expected);
  });
});
