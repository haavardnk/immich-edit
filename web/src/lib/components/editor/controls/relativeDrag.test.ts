import { describe, expect, it } from 'vitest';
import { FINE_FROM, MIN_FINE, dragPosition, fineFactor, valueAt } from './relativeDrag';
import { miredScale } from './sliderScale';

const TRACK = { min: -100, max: 100, width: 200 };

describe('relative slider drag', () => {
  it('moves by the finger travel, not to the finger', () => {
    expect(dragPosition(30, 10, 0, TRACK)).toBe(40);
    expect(dragPosition(30, -20, 0, TRACK)).toBe(10);
  });

  it('clamps to the track', () => {
    expect(dragPosition(95, 50, 0, TRACK)).toBe(100);
    expect(dragPosition(-95, -50, 0, TRACK)).toBe(-100);
  });

  it('slows down as the finger moves away from the track', () => {
    expect(fineFactor(FINE_FROM)).toBe(1);
    expect(fineFactor(-FINE_FROM - 80)).toBe(0.5);
    expect(fineFactor(10_000)).toBe(MIN_FINE);
    expect(dragPosition(0, 10, 120, TRACK)).toBe(5);
  });

  it('ignores a track with no width', () => {
    expect(dragPosition(12, 40, 0, { ...TRACK, width: 0 })).toBe(12);
  });

  it('maps a drag position back to a clamped value on linear and scaled tracks', () => {
    const linear = { min: -100, max: 100, step: 1, fineStep: 1, format: String };
    expect(valueAt(42.4, linear)).toBe(42);
    expect(valueAt(140, linear)).toBe(100);
    expect(valueAt(0.347, { ...linear, step: 0.05, fineStep: 0.01 })).toBe(0.35);
    const kelvin = { ...linear, min: 2000, max: 50000, scale: miredScale(2000, 50000) };
    expect(valueAt(0, kelvin)).toBe(2000);
  });
});
