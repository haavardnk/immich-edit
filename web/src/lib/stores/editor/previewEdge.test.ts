import { describe, expect, it } from 'vitest';
import { baseEdge, dragEdge, fitEdge, type ViewSnapshot } from './previewEdge';

function snap(width: number, height: number, viewW: number, viewH: number, dpr = 1): ViewSnapshot {
  return { frame: { left: 0, top: 0, width, height }, viewW, viewH, dpr };
}

describe('preview edges', () => {
  it.each([
    [null, 1600],
    [snap(500, 400, 500, 400), 1600],
    [snap(1000, 800, 1000, 800, 2), 2000],
    [snap(3000, 2000, 3000, 2000, 2), 4096]
  ])('baseEdge %#', (view, edge) => {
    expect(baseEdge(view)).toBe(edge);
  });

  it.each([
    [null, 1600],
    [snap(0, 0, 1000, 800), 1600],
    [snap(4000, 3000, 1000, 800, 2), 2000],
    [snap(1000, 800, 2000, 2000), 1600]
  ])('fitEdge %#', (view, edge) => {
    expect(fitEdge(view)).toBe(edge);
  });

  it.each([
    [null, 1600],
    [snap(4000, 3000, 1200, 900), 1200],
    [snap(4000, 3000, 3000, 2000), 1600]
  ])('dragEdge %#', (view, edge) => {
    expect(dragEdge(view)).toBe(edge);
  });
});
