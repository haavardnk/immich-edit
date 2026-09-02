import { describe, expect, it } from 'vitest';
import { splitPosition, viewportTransform, zoomAtAnchor } from './imageViewport';
import type { Rect } from './view-geometry';

describe('zoomAtAnchor', () => {
  const frame: Rect = { left: 100, top: 50, width: 400, height: 300 };

  it.each<[number, number, number, number]>([
    [100, 200, 0, -25],
    [100, 50, -150, -137.5]
  ])('anchors zoom %s to %s at the local cursor', (previous, next, panX, panY) => {
    expect(zoomAtAnchor(800, 600, frame, 200, 125, previous, next)).toEqual({ panX, panY });
  });
});

describe('splitPosition', () => {
  it.each<[number, number, number, number]>([
    [150, 100, 200, 0.25],
    [50, 100, 200, 0],
    [350, 100, 200, 1],
    [150, 100, 0, 0],
    [150, 100, -200, 0]
  ])('maps clientX %s in rect (%s, %s)', (clientX, left, width, expected) => {
    expect(splitPosition(clientX, left, width)).toBe(expected);
  });
});

describe('viewportTransform', () => {
  it.each<[number, number, number, string]>([
    [100, 0, 0, ''],
    [100, 20, -10, 'transform: scale(1) translate(20px, -10px); transform-origin: center;'],
    [200, 20, -10, 'transform: scale(2) translate(10px, -5px); transform-origin: center;']
  ])('formats zoom %s and pan (%s, %s)', (zoom, panX, panY, expected) => {
    expect(viewportTransform(zoom, panX, panY)).toBe(expected);
  });
});
