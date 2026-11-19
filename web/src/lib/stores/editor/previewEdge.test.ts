import { afterEach, describe, expect, it, vi } from 'vitest';
import { budgetFor, TOUCH_BUDGET } from '#lib/render/budget.ts';
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

describe('touch render budget', () => {
  afterEach(() => vi.unstubAllGlobals());

  it('applies only to the touch layout', () => {
    expect(budgetFor('touch')).toBe(TOUCH_BUDGET);
    expect(budgetFor('desktop')).not.toBe(TOUCH_BUDGET);
  });

  it('caps the device pixel ratio and the edges on touch', () => {
    const phone = snap(390, 260, 390, 600, 3);
    expect(baseEdge(phone, TOUCH_BUDGET)).toBe(1024);
    expect(baseEdge(snap(1600, 1000, 1600, 1000, 3), TOUCH_BUDGET)).toBe(2560);
    expect(largestEdge(phone, TOUCH_BUDGET)).toBe(1200);
    expect(dragEdge(snap(4000, 3000, 1200, 900), TOUCH_BUDGET)).toBe(1024);
    vi.stubGlobal('innerWidth', 390);
    vi.stubGlobal('innerHeight', 844);
    vi.stubGlobal('devicePixelRatio', 3);
    expect(windowEdge(TOUCH_BUDGET)).toBe(1688);
  });
});
