import { DEFAULT_THRESHOLDS } from '#lib/gestures/thresholds.ts';

export type StepDelta = 1 | -1;

export type DismissFrame = { offset: number; scale: number; fade: number };

export const SWIPE_GAP = 16;
const STEP_SHARE = 0.25;
const DISMISS_SHARE = 0.2;

const RUBBER = 0.55;
const MIN_SCALE = 0.7;
const FADE_SHARE = 0.5;

export function rubberBand(offset: number, extent: number): number {
  if (extent <= 0 || offset === 0) return 0;
  const pulled = (1 - 1 / ((Math.abs(offset) * RUBBER) / extent + 1)) * extent;
  return Math.sign(offset) * pulled;
}

export function deltaOf(dx: number): StepDelta {
  return dx < 0 ? 1 : -1;
}

export function followX(dx: number, width: number, canStep: (delta: StepDelta) => boolean): number {
  if (dx === 0 || canStep(deltaOf(dx))) return dx;
  return rubberBand(dx, width);
}

function flung(offset: number, velocity: number, fling: number): boolean {
  return Math.abs(velocity) >= fling && Math.sign(velocity) === Math.sign(offset);
}

export function swipeStep(
  dx: number,
  velocity: number,
  width: number,
  canStep: (delta: StepDelta) => boolean,
  fling = DEFAULT_THRESHOLDS.flingVelocity
): StepDelta | null {
  if (dx === 0) return null;
  const delta = deltaOf(dx);
  if (!canStep(delta)) return null;
  return Math.abs(dx) >= width * STEP_SHARE || flung(dx, velocity, fling) ? delta : null;
}

export function exitOffset(delta: StepDelta, width: number): number {
  return -delta * (width + SWIPE_GAP);
}

export function dismissFrame(dy: number, height: number): DismissFrame {
  if (dy <= 0) return { offset: rubberBand(dy, height), scale: 1, fade: 1 };
  const share = height > 0 ? Math.min(1, dy / height) : 0;
  return {
    offset: dy,
    scale: 1 - (1 - MIN_SCALE) * share,
    fade: Math.max(0, 1 - share / FADE_SHARE)
  };
}

export function dismisses(
  dy: number,
  velocity: number,
  height: number,
  fling = DEFAULT_THRESHOLDS.flingVelocity
): boolean {
  if (dy <= 0) return false;
  return dy >= height * DISMISS_SHARE || flung(dy, velocity, fling);
}
