import { describe, expect, it } from 'vitest';
import { WHEEL_PIXELS_PER_STEP, wheelSize, wheelStepper, type WheelSample } from './sliderWheel';

const KEYS = { shiftKey: false, altKey: false, ctrlKey: false, metaKey: false };

function sample(over: Partial<WheelSample>): WheelSample {
  return { ...KEYS, deltaX: 0, deltaY: 0, deltaMode: 0, timeStamp: 0, ...over };
}

describe('wheel size', () => {
  it.each([
    [KEYS, 'normal'],
    [{ ...KEYS, shiftKey: true }, 'coarse'],
    [{ ...KEYS, altKey: true }, 'fine'],
    [{ ...KEYS, ctrlKey: true }, null],
    [{ ...KEYS, shiftKey: true, altKey: true }, null]
  ] as const)('reads %o as %s', (keys, expected) => {
    expect(wheelSize(keys)).toBe(expected);
  });
});

describe('wheel stepper', () => {
  it('steps once per line or notch event, up raising the value', () => {
    const stepper = wheelStepper();
    expect(stepper.steps(sample({ deltaY: -3, deltaMode: 1, timeStamp: 10 }))).toBe(1);
    expect(stepper.steps(sample({ deltaY: 3, deltaMode: 1, timeStamp: 20 }))).toBe(-1);
    expect(stepper.steps(sample({ deltaY: -4, wheelDeltaY: 120, timeStamp: 30 }))).toBe(1);
    expect(stepper.steps(sample({ deltaY: 100, wheelDeltaY: -120, timeStamp: 40 }))).toBe(-1);
  });

  it('follows the dominant axis, as shift scrolls sideways on a Mac', () => {
    const stepper = wheelStepper();
    expect(stepper.steps(sample({ deltaX: -4, deltaY: 0, wheelDeltaX: 120, timeStamp: 10 }))).toBe(
      1
    );
  });

  it('accumulates smooth pixel deltas within a burst', () => {
    const stepper = wheelStepper();
    expect(stepper.steps(sample({ deltaY: -2, wheelDeltaY: 6, timeStamp: 0 }))).toBe(1);
    const burst = [16, 32, 48, 64, 80, 96, 112, 128].map((timeStamp) =>
      stepper.steps(sample({ deltaY: -WHEEL_PIXELS_PER_STEP / 4, wheelDeltaY: 30, timeStamp }))
    );
    expect(burst).toEqual([0, 0, 0, 1, 0, 0, 0, 1]);
  });

  it('starts a new burst with one step after a pause', () => {
    const stepper = wheelStepper();
    expect(stepper.steps(sample({ deltaY: 2, timeStamp: 0 }))).toBe(-1);
    expect(stepper.steps(sample({ deltaY: 2, timeStamp: 500 }))).toBe(-1);
    expect(stepper.steps(sample({ deltaY: 0, timeStamp: 510 }))).toBe(0);
  });
});
