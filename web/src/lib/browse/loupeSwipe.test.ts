import {
  dismisses,
  dismissFrame,
  exitOffset,
  followX,
  rubberBand,
  swipeStep,
  type StepDelta
} from '#lib/browse/loupeSwipe.ts';
import { describe, expect, it } from 'vitest';

const both = (): boolean => true;
const onlyNext = (delta: StepDelta): boolean => delta === 1;

describe('rubberBand', () => {
  it('resists more the further it is pulled and never reaches the extent', () => {
    const near = rubberBand(50, 400);
    const far = rubberBand(400, 400);
    expect(near).toBeGreaterThan(0);
    expect(near).toBeLessThan(50);
    expect(far).toBeLessThan(400 * 0.55);
    expect(rubberBand(10_000, 400)).toBeLessThan(400);
    expect(rubberBand(-50, 400)).toBeCloseTo(-near, 9);
    expect(rubberBand(0, 400)).toBe(0);
  });
});

describe('followX', () => {
  it('follows the finger towards a neighbour', () => {
    expect(followX(-120, 400, both)).toBe(-120);
  });

  it('rubber-bands towards a missing neighbour', () => {
    const pulled = followX(120, 400, onlyNext);
    expect(pulled).toBeGreaterThan(0);
    expect(pulled).toBeLessThan(120);
    expect(followX(-120, 400, onlyNext)).toBe(-120);
  });
});

describe('swipeStep', () => {
  it('steps to the next photo past a quarter of the width and back the other way', () => {
    expect(swipeStep(-100, 0, 400, both)).toBe(1);
    expect(swipeStep(100, 0, 400, both)).toBe(-1);
    expect(swipeStep(-99, 0, 400, both)).toBeNull();
  });

  it('steps on a fling in the drag direction', () => {
    expect(swipeStep(-30, -0.8, 400, both)).toBe(1);
    expect(swipeStep(-30, 0.8, 400, both)).toBeNull();
    expect(swipeStep(-30, -0.3, 400, both)).toBeNull();
  });

  it('never steps past an end', () => {
    expect(swipeStep(300, 2, 400, onlyNext)).toBeNull();
    expect(swipeStep(0, 2, 400, both)).toBeNull();
  });
});

describe('exitOffset', () => {
  it('moves the photo out by its width plus the gap against the step', () => {
    expect(exitOffset(1, 400)).toBe(-416);
    expect(exitOffset(-1, 400)).toBe(416);
  });
});

describe('dismissFrame', () => {
  it('shrinks the photo and fades the scrim as it is pulled down', () => {
    expect(dismissFrame(0, 800)).toEqual({ offset: 0, scale: 1, fade: 1 });
    const half = dismissFrame(200, 800);
    expect(half.offset).toBe(200);
    expect(half.scale).toBeCloseTo(0.925, 9);
    expect(half.fade).toBeCloseTo(0.5, 9);
    expect(dismissFrame(800, 800)).toEqual({ offset: 800, scale: 0.7, fade: 0 });
  });

  it('resists an upward pull without shrinking or fading', () => {
    const up = dismissFrame(-100, 800);
    expect(up.offset).toBeLessThan(0);
    expect(up.offset).toBeGreaterThan(-100);
    expect(up.scale).toBe(1);
    expect(up.fade).toBe(1);
  });
});

describe('dismisses', () => {
  it('closes past a fifth of the height or on a downward fling', () => {
    expect(dismisses(160, 0, 800)).toBe(true);
    expect(dismisses(159, 0, 800)).toBe(false);
    expect(dismisses(40, 0.9, 800)).toBe(true);
    expect(dismisses(40, -0.9, 800)).toBe(false);
    expect(dismisses(-200, -2, 800)).toBe(false);
  });
});
