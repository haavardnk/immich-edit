import { describe, expect, it } from 'vitest';
import { miredScale } from './sliderScale';
import { clampTo, entryText, nudgeValue, settleValue, snapToStep } from './sliderStep';

const EXPOSURE = { min: -5, max: 5, step: 0.05, fineStep: 0.01, format: String };
const PERCENT = { min: -100, max: 100, step: 1, fineStep: 1, format: String };
const KELVIN = {
  min: 2000,
  max: 50000,
  step: 1,
  fineStep: 1,
  format: String,
  scale: miredScale(2000, 50000)
};

describe('slider steps', () => {
  it('snaps to the step without float noise', () => {
    expect(snapToStep(0.349, -5, 0.05)).toBe(0.35);
    expect(snapToStep(12.4, -100, 1)).toBe(12);
    expect(snapToStep(-0.001, -100, 1)).toBe(0);
  });

  it('clamps without a negative zero', () => {
    expect(clampTo(-0, -1, 1)).toBe(0);
    expect(Object.is(clampTo(-0, -1, 1), -0)).toBe(false);
    expect(clampTo(9, -1, 1)).toBe(1);
  });

  it('settles a value onto the fine step inside the range', () => {
    expect(settleValue(0.347, EXPOSURE)).toBe(0.35);
    expect(settleValue(0.3449, EXPOSURE)).toBe(0.34);
    expect(settleValue(12, EXPOSURE)).toBe(5);
    expect(settleValue(-38.4, PERCENT)).toBe(-38);
  });

  it.each([
    ['normal', 1, 0.4],
    ['fine', 1, 0.36],
    ['coarse', 1, 0.85],
    ['normal', -2, 0.25],
    ['coarse', 10, 5]
  ] as const)('nudges a linear value by %s steps', (size, steps, expected) => {
    expect(nudgeValue(0.35, EXPOSURE, steps, size)).toBe(expected);
  });

  it('nudges a scaled value by track positions', () => {
    const at = (kelvin: number): number => Math.round(KELVIN.scale.toPosition(kelvin));
    expect(at(nudgeValue(5000, KELVIN, 1, 'normal'))).toBe(at(5000) + 1);
    expect(at(nudgeValue(5000, KELVIN, -1, 'coarse'))).toBe(at(5000) - 10);
    expect(nudgeValue(2000, KELVIN, -1, 'fine')).toBe(2000);
    expect(nudgeValue(50000, KELVIN, 3, 'normal')).toBe(50000);
  });

  it('writes entry text at the fine step without trailing zeros', () => {
    expect(entryText(0.35, EXPOSURE)).toBe('0.35');
    expect(entryText(1, EXPOSURE)).toBe('1');
    expect(entryText(-0, PERCENT)).toBe('0');
    expect(entryText(0.1 + 0.2, EXPOSURE)).toBe('0.3');
  });
});
