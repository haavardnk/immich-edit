import {
  scaleStepper,
  WHEEL_IDLE_MS,
  wheelStepper,
  wheelZoomAmount
} from '#lib/browse/gridZoom.ts';
import type { Point } from '#lib/gestures/point.ts';
import { held } from '#lib/input/modifiers.svelte.ts';
import { isPinchWheel } from '#lib/input/pinchWheel.ts';

export type GridZoomStep = (steps: number, at: Point) => void;

type SafariGesture = Event & { scale: number; clientX: number; clientY: number };

function isSafariGesture(e: Event): e is SafariGesture {
  return 'scale' in e && typeof e.scale === 'number';
}

export function listenGridZoom(element: HTMLElement, step: GridZoomStep): () => void {
  const wheelSteps = wheelStepper();
  const gestureSteps = scaleStepper();
  let touches = 0;
  let gesture: 'idle' | 'zoom' | 'touch' = 'idle';
  let gestureAt = Number.NEGATIVE_INFINITY;

  function inGesture(time: number): boolean {
    return gesture !== 'idle' && time - gestureAt <= WHEEL_IDLE_MS;
  }

  function onWheel(e: WheelEvent): void {
    if (!held('zoom', e)) return;
    e.preventDefault();
    if (inGesture(e.timeStamp)) return;
    const amount = wheelZoomAmount(e.deltaY, e.deltaMode);
    const steps = wheelSteps.push(amount, e.timeStamp, isPinchWheel(e));
    if (steps !== 0) step(steps, { x: e.clientX, y: e.clientY });
  }

  function onGestureStart(e: Event): void {
    e.preventDefault();
    gesture = touches > 0 ? 'touch' : 'zoom';
    gestureAt = e.timeStamp;
    gestureSteps.start();
  }

  function onGestureChange(e: Event): void {
    e.preventDefault();
    gestureAt = e.timeStamp;
    if (gesture !== 'zoom' || !isSafariGesture(e)) return;
    const steps = gestureSteps.to(e.scale);
    if (steps !== 0) step(steps, { x: e.clientX, y: e.clientY });
  }

  function onGestureEnd(e: Event): void {
    e.preventDefault();
    gesture = 'idle';
  }

  function onTouches(e: TouchEvent): void {
    touches = e.touches.length;
  }

  const listening = new AbortController();
  const options = { signal: listening.signal };
  element.addEventListener('wheel', onWheel, { ...options, passive: false });
  element.addEventListener('gesturestart', onGestureStart, options);
  element.addEventListener('gesturechange', onGestureChange, options);
  element.addEventListener('gestureend', onGestureEnd, options);
  for (const type of ['touchstart', 'touchend', 'touchcancel'] as const) {
    element.addEventListener(type, onTouches, { ...options, passive: true });
  }
  return () => {
    listening.abort();
    gesture = 'idle';
    touches = 0;
  };
}
