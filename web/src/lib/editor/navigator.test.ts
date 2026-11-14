import { viewportMetrics } from '#lib/utils/imageViewport.ts';
import { describe, expect, it } from 'vitest';
import { grabOffset, panTo, visibleBox } from './navigator';

const METRICS = viewportMetrics({ w: 400, h: 300 }, { w: 1000, h: 750 }, 1000, 1);

function rounded(values: readonly number[]): number[] {
  return values.map((value) => Math.round(value * 1000) / 1000);
}

describe('visibleBox', () => {
  it('covers the whole photo at fit', () => {
    expect(visibleBox(METRICS, { zoom: null, cx: 0.5, cy: 0.5 })).toEqual([0, 0, 1, 1]);
  });

  it('frames the visible part around the view centre', () => {
    expect(rounded(visibleBox(METRICS, { zoom: 100, cx: 0.5, cy: 0.5 }))).toEqual([
      0.3, 0.3, 0.4, 0.4
    ]);
  });

  it('stays inside the photo when the view is out of range', () => {
    expect(rounded(visibleBox(METRICS, { zoom: 100, cx: 0.95, cy: 0 }))).toEqual([
      0.6, 0, 0.4, 0.4
    ]);
  });
});

describe('grabOffset', () => {
  it('keeps the grab point when the press lands inside the box', () => {
    expect(rounded(Object.values(grabOffset([0.3, 0.3, 0.4, 0.4], { x: 0.6, y: 0.4 })))).toEqual([
      0.1, -0.1
    ]);
  });

  it('centres the box on a press outside it', () => {
    expect(grabOffset([0.3, 0.3, 0.4, 0.4], { x: 0.1, y: 0.9 })).toEqual({ x: 0, y: 0 });
  });
});

describe('panTo', () => {
  it('moves the view centre under the pointer minus the grab', () => {
    const view = panTo(
      METRICS,
      { zoom: 100, cx: 0.5, cy: 0.5 },
      { x: 0.7, y: 0.5 },
      { x: 0.1, y: 0 }
    );
    expect(rounded([view.cx, view.cy])).toEqual([0.6, 0.5]);
    expect(view.zoom).toBe(100);
  });

  it('clamps to the photo edge', () => {
    const view = panTo(METRICS, { zoom: 100, cx: 0.5, cy: 0.5 }, { x: 1, y: 0 }, { x: 0, y: 0 });
    expect(rounded([view.cx, view.cy])).toEqual([0.8, 0.2]);
  });
});
