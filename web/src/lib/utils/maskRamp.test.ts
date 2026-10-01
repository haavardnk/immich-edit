import { describe, expect, it } from 'vitest';
import { smoothRamp } from './maskRamp';

describe('smoothRamp', () => {
  it.each([
    [0, 0],
    [0.25, 0.15625],
    [0.5, 0.5],
    [0.75, 0.84375],
    [1, 1]
  ])('follows the render smoothstep at t=%f', (t, s) => {
    const stops = smoothRamp(0.2, 0.6, 0.1, 0.5);
    const stop = stops.find((x) => Math.abs(x.offset - (0.2 + 0.4 * t)) < 1e-9);
    expect(stop?.opacity).toBeCloseTo(0.1 + 0.4 * s, 9);
  });
});
