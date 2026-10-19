import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  baseEdge,
  dragEdge,
  fitEdge,
  largestEdge,
  windowEdge,
  type ViewSnapshot
} from './previewEdge';

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

describe('largest preview edge', () => {
  afterEach(() => vi.unstubAllGlobals());

  it('covers the whole viewer at device pixels', () => {
    const snap: ViewSnapshot = {
      frame: { left: 0, top: 0, width: 600, height: 400 },
      viewW: 1500,
      viewH: 900,
      dpr: 2
    };
    expect(largestEdge(snap)).toBe(3000);
  });

  it('reads the window before the viewer has measured itself', () => {
    vi.stubGlobal('innerWidth', 1440);
    vi.stubGlobal('innerHeight', 900);
    vi.stubGlobal('devicePixelRatio', 1.5);
    expect(windowEdge()).toBe(2160);
  });
});
