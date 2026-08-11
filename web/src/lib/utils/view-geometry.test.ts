import { describe, expect, it } from 'vitest';
import {
  frameBox,
  isFullFrame,
  placement,
  renderRequest,
  roiCovers,
  snapToDevice,
  visibleRegion,
  zoomAnchor,
  type Rect,
  type Roi
} from './view-geometry';

const SERVER_MAX = 8192;

function fitFrame(zoom: number, panX = 0, panY = 0): Rect {
  const frame = frameBox(1000, 800, 4000, 3200, zoom, panX, panY, 2);
  if (!frame) throw new Error('expected a frame');
  return frame;
}

function request(
  frame: Rect,
  srcLong: number,
  haveRoi: Roi | null = null,
  haveFullEdge = 0,
  serverMaxEdge = SERVER_MAX
) {
  const visible = visibleRegion(frame, 1000, 800);
  if (!visible) throw new Error('expected a visible region');
  return renderRequest({
    frame,
    visible,
    dpr: 2,
    srcLong,
    serverMaxEdge,
    haveRoi,
    haveFullEdge
  });
}

describe('snapToDevice', () => {
  it.each<[number, number, number]>([
    [10.3, 2, 10.5],
    [10.3, 1, 10],
    [10.26, 0, 10]
  ])('snaps %s at dpr %s', (value, dpr, expected) => {
    expect(snapToDevice(value, dpr)).toBe(expected);
  });
});

describe('frameBox', () => {
  it('centres the fitted frame at zoom 100', () => {
    expect(fitFrame(100)).toEqual({ left: 0, top: 0, width: 1000, height: 800 });
  });

  it('scales about the centre and applies pan', () => {
    expect(fitFrame(200, 30, -10)).toEqual({
      left: -470,
      top: -410,
      width: 2000,
      height: 1600
    });
  });

  it('snaps edges onto the device pixel grid', () => {
    const frame = frameBox(1001, 800, 4000, 3200, 100, 0, 0, 2);
    if (!frame) throw new Error('expected a frame');
    for (const v of [frame.left, frame.top, frame.width, frame.height]) {
      expect(Number.isInteger(v * 2)).toBe(true);
    }
  });

  it.each<[string, Parameters<typeof frameBox>]>([
    ['container not laid out', [0, 800, 4000, 3200, 100, 0, 0, 2]],
    ['source unknown', [1000, 800, 0, 3200, 100, 0, 0, 2]],
    ['zoom invalid', [1000, 800, 4000, 3200, 0, 0, 0, 2]]
  ])('returns null when %s', (_name, args) => {
    expect(frameBox(...args)).toBeNull();
  });
});

describe('visibleRegion', () => {
  it('is the whole frame when fitted', () => {
    expect(visibleRegion(fitFrame(100), 1000, 800)).toEqual([0, 0, 1, 1]);
  });

  it('is the centre quarter at 200%', () => {
    expect(visibleRegion(fitFrame(200), 1000, 800)).toEqual([0.25, 0.25, 0.5, 0.5]);
  });

  it('is null when panned off screen', () => {
    expect(visibleRegion(fitFrame(200, 3000), 1000, 800)).toBeNull();
  });
});

describe('renderRequest', () => {
  it('asks for the exact device size of the fitted frame', () => {
    expect(request(fitFrame(100), 4000)).toEqual({
      roi: [0, 0, 1, 1],
      maxEdge: 2000,
      fullEdge: 2000
    });
  });

  it('pads and quantizes the visible region when zoomed', () => {
    expect(request(fitFrame(200), 8000)).toEqual({
      roi: [102 / 512, 102 / 512, 308 / 512, 308 / 512],
      maxEdge: 2407,
      fullEdge: 4001
    });
  });

  it('never asks for more pixels than the source has', () => {
    expect(request(fitFrame(200), 4000)?.maxEdge).toBe(2406);
  });

  it('respects the server limit', () => {
    expect(request(fitFrame(200), 8000, null, 0, 2000)?.maxEdge).toBe(2000);
  });

  it('skips the request when the current render already covers it', () => {
    const tile = request(fitFrame(200), 8000);
    if (!tile) throw new Error('expected a request');
    expect(request(fitFrame(200), 8000, tile.roi, tile.fullEdge)).toBeNull();
  });

  it('asks again when zooming deeper needs more detail', () => {
    const near = request(fitFrame(200), 8000);
    if (!near) throw new Error('expected a request');
    const deep = request(fitFrame(400), 8000, near.roi, near.fullEdge);
    expect(deep?.fullEdge).toBe(7999);
  });

  it('stops asking once the source resolution is exhausted', () => {
    const near = request(fitFrame(200), 4000);
    if (!near) throw new Error('expected a request');
    expect(request(fitFrame(400), 4000, near.roi, near.fullEdge)).toBeNull();
  });
});

describe('placement', () => {
  it('takes its size from the delivered image, not the request', () => {
    expect(placement([0.25, 0, 0.5, 1], fitFrame(200), 1999, 3201, 2)).toEqual({
      left: 0,
      top: -400,
      width: 999.5,
      height: 1600.5
    });
  });

  it('falls back to the requested box when the source ran out of pixels', () => {
    expect(placement([0.25, 0, 0.5, 1], fitFrame(200), 1000, 1600, 2)).toEqual({
      left: 0,
      top: -400,
      width: 1000,
      height: 1600
    });
  });

  it('snaps the origin onto the device pixel grid', () => {
    const rect = placement([0.3333, 0.3333, 0.5, 0.5], fitFrame(200), 1000, 1000, 2);
    if (!rect) throw new Error('expected a placement');
    expect(Number.isInteger(rect.left * 2)).toBe(true);
    expect(Number.isInteger(rect.top * 2)).toBe(true);
  });
});

describe('zoomAnchor', () => {
  it('keeps the point under the cursor fixed', () => {
    const frame = fitFrame(100);
    const pan = zoomAnchor(1000, 800, frame, 250, 200, 2);
    const next = frameBox(1000, 800, 4000, 3200, 200, pan.panX, pan.panY, 2);
    if (!next) throw new Error('expected a frame');
    const u = (250 - frame.left) / frame.width;
    expect(next.left + u * next.width).toBeCloseTo(250, 6);
  });
});

describe('roiCovers and isFullFrame', () => {
  it('detects coverage and full frames', () => {
    expect(roiCovers([0, 0, 1, 1], [0.25, 0.25, 0.5, 0.5])).toBe(true);
    expect(roiCovers([0.3, 0.3, 0.2, 0.2], [0.25, 0.25, 0.5, 0.5])).toBe(false);
    expect(isFullFrame([0, 0, 1, 1])).toBe(true);
    expect(isFullFrame([0, 0, 0.9, 1])).toBe(false);
  });
});
