import { describe, expect, it } from 'vitest';
import {
  boxHandleHit,
  CROP_HANDLES,
  HANDLE_DOT,
  HANDLE_HIT,
  RETOUCH_SOURCE_HIT,
  spacedHandles
} from './handles.ts';

describe('handle hit areas', () => {
  it('gives every visible dot a hit area that contains it and meets 24 px', () => {
    for (const dot of Object.values(HANDLE_DOT)) {
      expect(dot).toBeLessThanOrEqual(HANDLE_HIT);
    }
    expect(HANDLE_HIT * 2).toBeGreaterThanOrEqual(24);
  });

  it('keeps the retouch source hit inside the shared hit radius', () => {
    expect(RETOUCH_SOURCE_HIT).toBeLessThanOrEqual(HANDLE_HIT);
  });
});

describe('spacedHandles', () => {
  it('uses the full hit radius for isolated handles', () => {
    const [a, b] = spacedHandles([
      { cx: 0, cy: 0, r: 5 },
      { cx: 100, cy: 0, r: 5 }
    ]);
    expect(a?.hit).toBe(HANDLE_HIT);
    expect(b?.hit).toBe(HANDLE_HIT);
  });

  it('caps the hit radius at half the distance to the nearest neighbour', () => {
    const spaced = spacedHandles([
      { cx: 0, cy: 0, r: 5 },
      { cx: 16, cy: 0, r: 5 },
      { cx: 0, cy: 40, r: 5 }
    ]);
    expect(spaced.map((h) => h.hit)).toEqual([8, 8, HANDLE_HIT]);
    const [a, b] = spaced;
    if (!a || !b) throw new Error('missing handles');
    expect(a.hit + b.hit).toBeLessThanOrEqual(Math.hypot(b.cx - a.cx, b.cy - a.cy));
  });

  it('never shrinks a hit area below its visible dot', () => {
    const spaced = spacedHandles([
      { cx: 0, cy: 0, r: 6 },
      { cx: 4, cy: 0, r: 5 }
    ]);
    expect(spaced.map((h) => h.hit)).toEqual([6, 5]);
  });

  it('keeps a preset hit radius and still counts it as a neighbour', () => {
    const spaced = spacedHandles([
      { cx: 0, cy: 0, r: 5, hit: 5 },
      { cx: 14, cy: 0, r: 7 }
    ]);
    expect(spaced.map((h) => h.hit)).toEqual([5, 7]);
  });

  it('keeps the descriptor fields and order', () => {
    const spaced = spacedHandles([
      { cx: 0, cy: 0, r: 5, label: 'first' },
      { cx: 50, cy: 0, r: 5, label: 'last' }
    ]);
    expect(spaced.map((h) => h.label)).toEqual(['first', 'last']);
  });
});

describe('crop handles', () => {
  it('scales the hit size with the crop and clamps it', () => {
    expect(boxHandleHit(400, 300)).toBe(HANDLE_HIT);
    expect(boxHandleHit(40, 200)).toBe(10);
    expect(boxHandleHit(8, 8)).toBe(HANDLE_DOT.crop);
  });

  it('leaves the middle of the crop free for moving', () => {
    for (const size of [24, 40, 48, 60, 100]) {
      expect(boxHandleHit(size, size) * 2).toBeLessThanOrEqual(size / 2);
    }
  });

  it('renders every edge before any corner so corners win overlaps', () => {
    const lastEdge = Math.max(
      ...(['n', 'e', 's', 'w'] as const).map((h) => CROP_HANDLES.indexOf(h))
    );
    const firstCorner = Math.min(
      ...(['nw', 'ne', 'se', 'sw'] as const).map((h) => CROP_HANDLES.indexOf(h))
    );
    expect(new Set(CROP_HANDLES).size).toBe(8);
    expect(lastEdge).toBeLessThan(firstCorner);
  });
});
