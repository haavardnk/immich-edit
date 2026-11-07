import type { Point } from '#lib/gestures/point.ts';
import { describe, expect, it } from 'vitest';
import {
  clampView,
  FIT_VIEW,
  frameOffset,
  frameSize,
  isZoomed,
  panByPixels,
  panByStep,
  panStep,
  pinchView,
  scaleOf,
  viewportMetrics,
  viewportTransform,
  wheelZoom,
  withZoom,
  zoomAround,
  zoomRatio,
  type View,
  type ViewportMetrics
} from './imageViewport';
import { fitScale, frameBox, nativeScale } from './viewGeometry';

const METRICS = viewportMetrics({ w: 800, h: 600 }, { w: 800, h: 600 }, 1600, 1);
const WIDE = viewportMetrics({ w: 800, h: 600 }, { w: 800, h: 200 }, 1600, 1);

function at(zoom: number | null, cx = 0.5, cy = 0.5): View {
  return { zoom, cx, cy };
}

function imagePoint(metrics: ViewportMetrics, view: View, anchor: Point): Point {
  const frame = frameSize(metrics, view.zoom);
  return { x: view.cx + anchor.x / frame.w, y: view.cy + anchor.y / frame.h };
}

describe('viewportMetrics', () => {
  it('measures fit, one source pixel and the fit zoom', () => {
    expect(METRICS).toEqual({
      box: { w: 800, h: 600 },
      natural: { w: 800, h: 600 },
      fit: 1,
      unit: 2,
      fitZoom: 50
    });
  });

  it('keeps the fit zoom unrounded', () => {
    const metrics = viewportMetrics({ w: 997, h: 700 }, { w: 1000, h: 667 }, 6000, 1.5);
    const fit = fitScale(997, 700, 1000, 667);
    const unit = nativeScale(1000, 6000, 1.5);
    expect(metrics.fitZoom).toBe((100 * fit) / unit);
    expect(Number.isInteger(metrics.fitZoom)).toBe(false);
  });

  it('treats fit as 100% while the source size is unknown', () => {
    const metrics = viewportMetrics({ w: 800, h: 600 }, { w: 400, h: 300 }, 0, 1);
    expect(metrics.unit).toBe(metrics.fit);
    expect(metrics.fitZoom).toBe(100);
  });

  it('reports 100% before the image has a size', () => {
    const metrics = viewportMetrics({ w: 800, h: 600 }, { w: 0, h: 0 }, 1600, 1);
    expect(metrics).toMatchObject({ fit: 0, unit: 0, fitZoom: 100 });
  });
});

describe('scale and frame', () => {
  it.each<[number | null, number, number, number]>([
    [null, 1, 800, 1],
    [50, 1, 800, 1],
    [100, 2, 1600, 2],
    [200, 4, 3200, 4]
  ])('zoom %s scales by %s', (zoom, scale, width, ratio) => {
    expect(scaleOf(METRICS, zoom)).toBe(scale);
    expect(frameSize(METRICS, zoom).w).toBe(width);
    expect(zoomRatio(METRICS, zoom)).toBe(ratio);
  });

  it('keeps the ratio at one before the box is measured', () => {
    expect(zoomRatio(viewportMetrics({ w: 0, h: 0 }, { w: 0, h: 0 }, 0, 1), 200)).toBe(1);
  });

  it.each<[View, boolean]>([
    [FIT_VIEW, false],
    [at(50), false],
    [at(40), false],
    [at(51), true]
  ])('%j is zoomed: %s', (view, zoomed) => {
    expect(isZoomed(METRICS, view)).toBe(zoomed);
  });
});

describe('clampView', () => {
  it.each<[View, View]>([
    [at(100, 0.6, 0.4), at(100, 0.6, 0.4)],
    [at(100, 0.9, 0.1), at(100, 0.75, 0.25)],
    [at(100, 0, 1), at(100, 0.25, 0.75)],
    [at(200, 0, 1), at(200, 0.125, 0.875)],
    [at(null, 0.1, 0.9), FIT_VIEW],
    [at(30, 0.1, 0.9), at(30)]
  ])('limits %j to %j', (view, expected) => {
    expect(clampView(METRICS, view)).toEqual(expected);
  });

  it('centres an axis the photo does not fill', () => {
    expect(clampView(WIDE, at(100, 0.9, 0.9))).toEqual(at(100, 0.75, 0.5));
  });

  it('leaves the view alone before anything is measured', () => {
    const unknown = viewportMetrics({ w: 0, h: 0 }, { w: 800, h: 600 }, 1600, 1);
    expect(clampView(unknown, at(100, 0.9, 0.1))).toEqual(at(100, 0.9, 0.1));
  });
});

describe('withZoom', () => {
  it('keeps the centre above fit', () => {
    expect(withZoom(at(100, 0.3, 0.7), 200, 50)).toEqual(at(200, 0.3, 0.7));
  });

  it('sets a zoom on a fit view', () => {
    expect(withZoom(at(null, 0.3, 0.7), 200, 50)).toEqual(at(200, 0.3, 0.7));
  });

  it.each<[number, number]>([
    [50, 50],
    [30, 30],
    [10, 25]
  ])('centres a zoom of %s that is not above fit at %s', (zoom, expected) => {
    expect(withZoom(at(200, 0.7, 0.3), zoom, 50)).toEqual(at(expected));
  });

  it('caps the zoom', () => {
    expect(withZoom(at(200, 0.7, 0.3), 5000, 50)).toEqual(at(800, 0.7, 0.3));
  });
});

describe('zoomAround', () => {
  it.each<[View, number, Point]>([
    [FIT_VIEW, 100, { x: 200, y: 150 }],
    [at(100), 200, { x: 100, y: 50 }],
    [at(200, 0.4, 0.6), 100, { x: -50, y: 80 }],
    [at(100, 0.6, 0.45), 300, { x: 0, y: 0 }]
  ])('keeps the point under the anchor from %j to %s at %j', (view, zoom, anchor) => {
    const next = zoomAround(METRICS, view, zoom, anchor);
    expect(next.zoom).toBe(zoom);
    const before = imagePoint(METRICS, view, anchor);
    const after = imagePoint(METRICS, next, anchor);
    expect(after.x).toBeCloseTo(before.x, 9);
    expect(after.y).toBeCloseTo(before.y, 9);
  });

  it('anchors a zoom from fit at the pointer', () => {
    expect(zoomAround(METRICS, FIT_VIEW, 100, { x: 200, y: -150 })).toEqual(at(100, 0.625, 0.375));
  });

  it('clamps the result to the photo edge', () => {
    expect(zoomAround(METRICS, FIT_VIEW, 100, { x: 400, y: 300 })).toEqual(at(100, 0.75, 0.75));
    expect(zoomAround(METRICS, at(100, 0.75, 0.75), 200, { x: 400, y: 300 })).toEqual(
      at(200, 0.875, 0.875)
    );
  });

  it('starts from the shown view', () => {
    expect(zoomAround(METRICS, at(100, 0.95, 0.5), 100, { x: 0, y: 0 })).toEqual(
      at(100, 0.75, 0.5)
    );
  });

  it.each<[number, number]>([
    [5000, 800],
    [800, 800]
  ])('caps zoom %s at %s', (zoom, expected) => {
    expect(zoomAround(METRICS, at(400), zoom, { x: 0, y: 0 }).zoom).toBe(expected);
  });

  it.each<[number, number]>([
    [50, 50],
    [30, 30],
    [10, 25]
  ])('centres a zoom of %s that is not above fit at %s', (zoom, expected) => {
    expect(zoomAround(METRICS, at(200, 0.7, 0.3), zoom, { x: 100, y: 100 })).toEqual(at(expected));
  });

  it('keeps the centre when the frame is unknown', () => {
    const unknown = viewportMetrics({ w: 800, h: 600 }, { w: 0, h: 0 }, 1600, 1);
    expect(zoomAround(unknown, at(150, 0.3, 0.6), 200, { x: 50, y: 50 })).toEqual(
      at(200, 0.3, 0.6)
    );
  });
});

describe('pinchView', () => {
  it('scales the start zoom about the first focal point', () => {
    const anchor = { x: 100, y: 50 };
    expect(pinchView(METRICS, FIT_VIEW, 2, anchor, { x: 0, y: 0 })).toEqual(
      zoomAround(METRICS, FIT_VIEW, 100, anchor)
    );
    expect(pinchView(METRICS, at(100), 1.5, anchor, { x: 0, y: 0 }).zoom).toBe(150);
  });

  it('keeps the pinched point under the fingers as they move', () => {
    const anchor = { x: 100, y: 50 };
    const next = pinchView(METRICS, FIT_VIEW, 2, anchor, { x: 40, y: -20 });
    const before = imagePoint(METRICS, FIT_VIEW, anchor);
    const after = imagePoint(METRICS, next, { x: 140, y: 30 });
    expect(after.x).toBeCloseTo(before.x, 9);
    expect(after.y).toBeCloseTo(before.y, 9);
  });

  it('centres a pinch that ends below fit', () => {
    expect(pinchView(METRICS, FIT_VIEW, 0.8, { x: 100, y: 50 }, { x: 30, y: 0 })).toEqual(at(40));
  });
});

describe('wheelZoom', () => {
  it.each<[number, number, number]>([
    [100, -1, 110],
    [110, 1, 100]
  ])('steps %s by wheel delta %s to %s', (zoom, deltaY, expected) => {
    expect(wheelZoom(zoom, deltaY)).toBeCloseTo(expected, 9);
  });
});

describe('panByPixels', () => {
  it.each<[View, number, number, View]>([
    [at(100), 160, 0, at(100, 0.4, 0.5)],
    [at(100), 0, -120, at(100, 0.5, 0.6)],
    [at(100), 1600, 1200, at(100, 0.25, 0.25)],
    [at(100, 0.95, 0.5), -160, 0, at(100, 0.75, 0.5)],
    [at(100, 0.95, 0.5), 160, 0, at(100, 0.65, 0.5)],
    [FIT_VIEW, 300, 300, FIT_VIEW]
  ])('moves %j by (%s, %s) to %j', (view, dx, dy, expected) => {
    const next = panByPixels(METRICS, view, dx, dy);
    expect(next.zoom).toBe(expected.zoom);
    expect(next.cx).toBeCloseTo(expected.cx, 9);
    expect(next.cy).toBeCloseTo(expected.cy, 9);
  });

  it('keeps the view when the frame is unknown', () => {
    const unknown = viewportMetrics({ w: 800, h: 600 }, { w: 0, h: 0 }, 1600, 1);
    expect(panByPixels(unknown, at(200, 0.3, 0.6), 50, 50)).toEqual(at(200, 0.3, 0.6));
  });
});

describe('panByStep', () => {
  it.each<[string, View, View]>([
    ['ArrowRight', at(100), at(100, 0.55, 0.5)],
    ['ArrowLeft', at(100), at(100, 0.45, 0.5)],
    ['ArrowDown', at(100), at(100, 0.5, 0.55)],
    ['ArrowUp', at(100), at(100, 0.5, 0.45)],
    ['ArrowRight', at(100, 0.74, 0.5), at(100, 0.75, 0.5)],
    ['ArrowUp', at(100, 0.5, 0.1), at(100, 0.5, 0.25)]
  ])('%s moves %j to %j within the photo', (key, view, expected) => {
    const step = panStep(key);
    if (!step) throw new Error('no step');
    const next = panByStep(METRICS, view, step);
    expect(next.cx).toBeCloseTo(expected.cx, 9);
    expect(next.cy).toBeCloseTo(expected.cy, 9);
  });

  it('has no step for other keys', () => {
    expect(panStep('Home')).toBeNull();
  });
});

describe('frameOffset', () => {
  it.each<[View, Point]>([
    [FIT_VIEW, { x: 0, y: 0 }],
    [at(100, 0.25, 0.75), { x: 400, y: -300 }],
    [at(200, 0.6, 0.5), { x: -320, y: 0 }]
  ])('offsets %j by %j', (view, expected) => {
    const offset = frameOffset(METRICS, view);
    expect(offset.x).toBeCloseTo(expected.x, 9);
    expect(offset.y).toBeCloseTo(expected.y, 9);
  });

  it('places the view centre at the container centre through frameBox', () => {
    const view = at(100, 0.3, 0.6);
    const offset = frameOffset(METRICS, view);
    const frame = frameBox(800, 600, 800, 600, scaleOf(METRICS, 100), offset.x, offset.y, 1);
    if (!frame) throw new Error('expected a frame');
    expect(frame.left + view.cx * frame.width).toBeCloseTo(400, 6);
    expect(frame.top + view.cy * frame.height).toBeCloseTo(300, 6);
  });
});

describe('viewportTransform', () => {
  it.each<[View, string]>([
    [FIT_VIEW, ''],
    [at(50), ''],
    [at(100), 'transform: scale(2) translate(0px, 0px); transform-origin: center;'],
    [
      at(100, 0.25, 0.75),
      'transform: scale(2) translate(200px, -150px); transform-origin: center;'
    ],
    [at(25), 'transform: scale(0.5) translate(0px, 0px); transform-origin: center;']
  ])('formats %j', (view, expected) => {
    expect(viewportTransform(METRICS, view)).toBe(expected);
  });
});
