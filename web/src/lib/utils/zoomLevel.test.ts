import { describe, expect, it } from 'vitest';
import {
  clampZoom,
  isFitZoom,
  MAX_ZOOM,
  nextStop,
  SLIDER_STEPS,
  sliderToZoom,
  zoomStops,
  zoomToSlider
} from './zoomLevel';

describe('clampZoom', () => {
  it.each<[string, number, number, number]>([
    ['floors at half of fit', 5, 25, 12.5],
    ['allows zooming out below fit', 20, 25, 20],
    ['never floors above 1:1', 50, 400, 100],
    ['caps at the maximum', 5000, 25, MAX_ZOOM],
    ['raises the cap so a magnifying fit stays reachable', 5000, 1560, 1560],
    ['leaves values inside the range alone', 150, 25, 150]
  ])('%s', (_name, zoom, fitZoom, expected) => {
    expect(clampZoom(zoom, fitZoom)).toBe(expected);
  });
});

describe('zoomStops', () => {
  it('runs from half of fit through fit', () => {
    expect(zoomStops(30)).toEqual([15, 25, 30, 33, 50, 66, 100, 200, 300, 400, MAX_ZOOM]);
  });

  it('keeps 1:1 reachable when fit magnifies', () => {
    expect(zoomStops(250)).toEqual([100, 200, 250, 300, 400, MAX_ZOOM]);
  });

  it('lets fit replace a stop it sits on', () => {
    expect(zoomStops(50.2)).toEqual([25.1, 33, 50.2, 66, 100, 200, 300, 400, MAX_ZOOM]);
  });
});

describe('nextStop', () => {
  it.each<[number, 1 | -1, number]>([
    [25, 1, 33],
    [100, 1, 200],
    [100, -1, 66],
    [26, -1, 25],
    [25, -1, 12.5],
    [13, 1, 25],
    [MAX_ZOOM, 1, MAX_ZOOM],
    [12.5, -1, 12.5]
  ])('steps %s in direction %s', (zoom, direction, expected) => {
    expect(nextStop(zoom, direction, 25)).toBe(expected);
  });
});

describe('isFitZoom', () => {
  it('treats a value within half a percent as fit', () => {
    expect(isFitZoom(25.4, 25)).toBe(true);
    expect(isFitZoom(26, 25)).toBe(false);
  });
});

describe('slider mapping', () => {
  it('anchors both ends of the track', () => {
    expect(zoomToSlider(12.5, 25)).toBe(0);
    expect(zoomToSlider(MAX_ZOOM, 25)).toBe(SLIDER_STEPS);
  });

  it('gives equal travel to equal magnification ratios', () => {
    const fit = zoomToSlider(25, 25);
    const double = zoomToSlider(50, 25);
    const quadruple = zoomToSlider(100, 25);
    expect(Math.abs(quadruple - double - (double - fit))).toBeLessThanOrEqual(1);
  });

  it('round-trips through the log scale', () => {
    expect(sliderToZoom(zoomToSlider(200, 25), 25)).toBeCloseTo(200, 0);
  });
});
