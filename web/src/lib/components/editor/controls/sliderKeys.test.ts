import { describe, expect, it } from 'vitest';
import { arrowStepValue } from './sliderKeys';

const NONE = { key: 'ArrowRight', shiftKey: false, altKey: false, ctrlKey: false, metaKey: false };
const SHIFT = { ...NONE, shiftKey: true };
const ALT = { ...NONE, altKey: true };
const EXPOSURE = { min: -5, max: 5, step: 0.05, fineStep: 0.01, coarseStep: 0.5 };
const PERCENT = { min: -100, max: 100, step: 1, fineStep: 1, coarseStep: 10 };

describe('arrowStepValue', () => {
  it.each([
    ['ArrowRight', 0, 0.05],
    ['ArrowUp', 0, 0.05],
    ['ArrowLeft', 0.35, 0.3],
    ['ArrowDown', 0.35, 0.3]
  ])('plain %s moves %s one step to %s', (key, value, expected) => {
    expect(arrowStepValue({ ...NONE, key }, { ...EXPOSURE, value })).toBe(expected);
  });

  it.each([
    ['ArrowRight', 0.35, 0.85],
    ['ArrowLeft', 0.35, -0.15]
  ])('shift %s moves %s ten steps to %s', (key, value, expected) => {
    expect(arrowStepValue({ ...SHIFT, key }, { ...EXPOSURE, value })).toBe(expected);
  });

  it.each([
    ['ArrowRight', 0.35, 0.36],
    ['ArrowLeft', 0.35, 0.34]
  ])('alt %s moves %s one fine step to %s', (key, value, expected) => {
    expect(arrowStepValue({ ...ALT, key }, { ...EXPOSURE, value })).toBe(expected);
  });

  it.each([
    ['clamps at the maximum', 'ArrowRight', 96, 100],
    ['clamps at the minimum', 'ArrowLeft', -96, -100]
  ])('%s', (_name, key, value, expected) => {
    expect(arrowStepValue({ ...SHIFT, key }, { ...PERCENT, value })).toBe(expected);
  });

  it.each([
    ['shift with alt is no step', { shiftKey: true, altKey: true }],
    ['ctrl belongs to the browser', { ctrlKey: true }],
    ['meta belongs to the browser', { metaKey: true }],
    ['non-arrow keys are ignored', { key: 'Home' }]
  ])('%s', (_name, event) => {
    expect(arrowStepValue({ ...NONE, ...event }, { ...PERCENT, value: 0 })).toBeNull();
  });
});
