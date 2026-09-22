import { describe, expect, it } from 'vitest';
import { arrowStepValue } from './sliderKeys';

const base = { key: 'ArrowRight', shiftKey: true, altKey: false, ctrlKey: false, metaKey: false };

describe('arrowStepValue', () => {
  it.each([
    ['ArrowRight', 1, 10, 0, 10],
    ['ArrowUp', 1, 10, 0, 10],
    ['ArrowLeft', 1, 10, 0, -10],
    ['ArrowDown', 1, 10, 0, -10],
    ['ArrowRight', 0.05, 0.5, 0.35, 0.85],
    ['ArrowLeft', 0.05, 0.5, 0.35, -0.15],
    ['ArrowRight', 0.01, 0.1, 0.07, 0.17]
  ])('%s with step %s moves %s to %s', (key, step, coarseStep, value, expected) => {
    expect(arrowStepValue({ ...base, key }, { value, min: -100, max: 100, step, coarseStep })).toBe(
      expected
    );
  });

  it.each([
    ['clamps at the maximum', 'ArrowRight', 96, 100],
    ['clamps at the minimum', 'ArrowLeft', -96, -100]
  ])('%s', (_name, key, value, expected) => {
    expect(
      arrowStepValue({ ...base, key }, { value, min: -100, max: 100, step: 1, coarseStep: 10 })
    ).toBe(expected);
  });

  it.each([
    ['plain arrow keys stay native', { shiftKey: false }],
    ['alt is reserved for preview drag', { altKey: true }],
    ['ctrl belongs to the browser', { ctrlKey: true }],
    ['meta belongs to the browser', { metaKey: true }],
    ['non-arrow keys are ignored', { key: 'Home' }]
  ])('%s', (_name, event) => {
    expect(
      arrowStepValue(
        { ...base, ...event },
        { value: 0, min: -100, max: 100, step: 1, coarseStep: 10 }
      )
    ).toBeNull();
  });
});
