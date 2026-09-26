import { describe, expect, it } from 'vitest';
import { nudgePan, panStep, splitPosition, viewportTransform, zoomAtAnchor } from './imageViewport';
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
    [1, 0, 0, ''],
    [1, 20, -10, 'transform: scale(1) translate(20px, -10px); transform-origin: center;'],
    [2, 20, -10, 'transform: scale(2) translate(10px, -5px); transform-origin: center;']
  ])('formats fit ratio %s and pan (%s, %s)', (fitRatio, panX, panY, expected) => {
    expect(viewportTransform(fitRatio, panX, panY)).toBe(expected);
  });
});

describe('nudgePan', () => {
  const frame = { width: 2000, height: 1000 };

  it.each<[string, number, number, number, number]>([
    ['ArrowRight', 0, 0, -80, 0],
    ['ArrowLeft', 0, 0, 80, 0],
    ['ArrowDown', 0, 0, 0, -60],
    ['ArrowUp', 0, 0, 0, 60],
    ['ArrowRight', -560, 0, -600, 0],
    ['ArrowUp', 0, 180, 0, 200]
  ])('%s from (%s, %s) moves to (%s, %s) within the frame', (key, x, y, panX, panY) => {
    const step = panStep(key);
    if (!step) throw new Error('no step');
    expect(nudgePan({ panX: x, panY: y }, step, frame, 800, 600)).toEqual({ panX, panY });
  });

  it('has no step for other keys', () => {
    expect(panStep('Home')).toBeNull();
  });
});
