import { describe, expect, it } from 'vitest';
import { windowLevels } from './depthWindow';

const ramp = [255, 170, 85, 0];

describe('windowLevels', () => {
  it.each([
    ['full window keeps a flat map', [128, 128, 128, 128], 0, 1, 0, [255, 255, 255, 255]],
    ['far limit drops the farthest level first', ramp, 0, 0.8, 0, [255, 255, 255, 0]],
    ['near limit drops the nearest level first', ramp, 0.3, 1, 0, [0, 255, 255, 255]],
    [
      'meeting handles keep the level they fall in',
      [255, 0, 0, 0],
      0.5,
      0.5,
      0,
      [0, 255, 255, 255]
    ],
    ['a window inside one level keeps that level', [255, 0, 0, 0], 0.4, 0.6, 0, [0, 255, 255, 255]],
    ['softness fades the level just past the far limit', ramp, 0, 0.5, 0.5, [255, 255, 128, 0]]
  ])('%s', (_name, levels, min, max, softness, want) => {
    expect(Array.from(windowLevels(new Uint8Array(levels), { min, max, softness }))).toEqual(want);
  });
});
