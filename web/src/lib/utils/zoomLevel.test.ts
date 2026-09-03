import { describe, expect, it } from 'vitest';
import {
  clampZoom,
  MAX_ZOOM,
  nextStop,
  SLIDER_STEPS,
  sliderToZoom,
  zoomStops,
  zoomToSlider
} from './zoomLevel';

describe('clampZoom', () => {
  it.each<[string, number, number, number]>([
    ['floors at fit when fit magnifies down', 5, 25, 25],
    ['never floors above 1:1', 50, 400, 100],
    ['caps at the maximum', 5000, 25, MAX_ZOOM],
    ['raises the cap so a magnifying fit stays reachable', 5000, 1560, 1560],
    ['leaves values inside the range alone', 150, 25, 150]
  ])('%s', (_name, zoom, fitZoom, expected) => {
    expect(clampZoom(zoom, fitZoom)).toBe(expected);
  });
});

describe('zoomStops', () => {
  it('starts at fit and drops stops it swallows', () => {
    expect(zoomStops(30)).toEqual([30, 33, 50, 66, 100, 200, 300, 400, MAX_ZOOM]);
  });

  it('keeps 1:1 reachable when fit magnifies', () => {
    expect(zoomStops(250)).toEqual([100, 200, 300, 400, MAX_ZOOM]);
  });
});

describe('nextStop', () => {
  it.each<[number, 1 | -1, number]>([
    [25, 1, 33],
    [100, 1, 200],
    [100, -1, 66],
    [26, -1, 25],
    [MAX_ZOOM, 1, MAX_ZOOM],
    [25, -1, 25]
  ])('steps %s in direction %s', (zoom, direction, expected) => {
    expect(nextStop(zoom, direction, 25)).toBe(expected);
  });
});

describe('slider mapping', () => {
  it('anchors both ends of the track', () => {
    expect(zoomToSlider(25, 25)).toBe(0);
    expect(zoomToSlider(MAX_ZOOM, 25)).toBe(SLIDER_STEPS);
  });

  it('gives equal travel to equal magnification ratios', () => {
    const quarter = zoomToSlider(50, 25);
    expect(zoomToSlider(100, 25) - quarter).toBe(quarter);
  });

  it('round-trips through the log scale', () => {
    expect(sliderToZoom(zoomToSlider(200, 25), 25)).toBeCloseTo(200, 1);
  });
});
