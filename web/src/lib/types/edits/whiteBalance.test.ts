import { describe, expect, it } from 'vitest';
import { miredScale } from '$lib/components/editor/controls/sliderScale';
import {
  absoluteWhiteBalance,
  asShotWhiteBalance,
  relativeShift,
  resolveTemperature,
  type WhiteBalance
} from './whiteBalance';

const SHOT = { kelvin: 5000, tint: 10 };

describe('white balance', () => {
  it.each<[WhiteBalance, number, number]>([
    [asShotWhiteBalance(), 5000, 10],
    [{ mode: 'relative', temp: 100, tint: 100 }, 5000 * 2 ** 1.5, 100],
    [{ mode: 'relative', temp: -100, tint: -100 }, 2000, -80],
    [{ mode: 'absolute', kelvin: 60000, tint: -200 }, 50000, -150]
  ])('resolves %o against the as-shot white', (wb, kelvin, tint) => {
    const white = resolveTemperature(wb, SHOT);
    expect(white.kelvin).toBeCloseTo(kelvin, 6);
    expect(white.tint).toBeCloseTo(tint, 6);
  });

  it('inverts a relative shift through an absolute white', () => {
    const white = resolveTemperature({ mode: 'relative', temp: 30, tint: -10 }, SHOT);
    const shift = relativeShift({ mode: 'absolute', ...white }, SHOT);
    expect(shift.temp).toBeCloseTo(30, 6);
    expect(shift.tint).toBeCloseTo(-10, 6);
  });

  it.each<[number, number, WhiteBalance]>([
    [5000.4, 9.6, asShotWhiteBalance()],
    [3199.6, -4.4, { mode: 'absolute', kelvin: 3200, tint: -4 }]
  ])('stores %d K tint %d in native form', (kelvin, tint, want) => {
    expect(absoluteWhiteBalance({ kelvin, tint }, SHOT)).toEqual(want);
  });

  it('maps kelvin linearly in mired', () => {
    const scale = miredScale(2000, 50000);
    expect(scale.toPosition(2000)).toBeCloseTo(0, 6);
    expect(scale.toPosition(50000)).toBeCloseTo(1000, 6);
    expect(scale.toPosition(5000)).toBeCloseTo(625, 6);
    expect(scale.fromPosition(scale.toPosition(6500))).toBe(6500);
  });
});
