import { heldAlone, unmodified, type ModifierKeys } from '#lib/input/modifiers.svelte.ts';
import type { Attachment } from 'svelte/attachments';
import type { StepSize } from './sliderStep';

export const WHEEL_REST_MS = 300;
export const WHEEL_SETTLE_MS = 400;
export const WHEEL_PIXELS_PER_STEP = 40;
const BURST_GAP_MS = 150;
const NOTCH = 120;

export type WheelSample = ModifierKeys &
  Pick<WheelEvent, 'deltaX' | 'deltaY' | 'deltaMode' | 'timeStamp'> & {
    wheelDeltaX?: number;
    wheelDeltaY?: number;
  };

export type SliderWheelSpec = {
  enabled: () => boolean;
  nudge: (steps: number, size: StepSize) => void;
  settle: () => void;
};

export function wheelSize(e: ModifierKeys): StepSize | null {
  if (heldAlone('coarse', e)) return 'coarse';
  if (heldAlone('fine', e)) return 'fine';
  return unmodified(e) ? 'normal' : null;
}

function dominant(e: WheelSample): { delta: number; legacy: number | undefined } {
  return Math.abs(e.deltaX) > Math.abs(e.deltaY)
    ? { delta: e.deltaX, legacy: e.wheelDeltaX }
    : { delta: e.deltaY, legacy: e.wheelDeltaY };
}

export function wheelStepper(): { steps: (e: WheelSample) => number; reset: () => void } {
  let pixels = 0;
  let last = -Infinity;

  function steps(e: WheelSample): number {
    const { delta, legacy } = dominant(e);
    const gap = e.timeStamp - last;
    last = e.timeStamp;
    if (delta === 0) return 0;
    const direction = delta < 0 ? 1 : -1;
    const notch = legacy !== undefined && legacy !== 0 && legacy % NOTCH === 0;
    if (e.deltaMode !== 0 || notch || gap > BURST_GAP_MS) {
      pixels = 0;
      return direction;
    }
    pixels -= delta;
    const whole = Math.trunc(pixels / WHEEL_PIXELS_PER_STEP);
    pixels -= whole * WHEEL_PIXELS_PER_STEP;
    return whole;
  }

  function reset(): void {
    pixels = 0;
    last = -Infinity;
  }

  return { steps, reset };
}

export function sliderWheel(spec: SliderWheelSpec): Attachment<HTMLElement> {
  return (element) => {
    const stepper = wheelStepper();
    let engaged = false;
    let adjusting = false;
    let rest: ReturnType<typeof setTimeout> | undefined;
    let idle: ReturnType<typeof setTimeout> | undefined;

    function wait(): void {
      clearTimeout(rest);
      rest = setTimeout(() => {
        engaged = true;
      }, WHEEL_REST_MS);
    }

    function finish(): void {
      clearTimeout(idle);
      stepper.reset();
      if (!adjusting) return;
      adjusting = false;
      spec.settle();
    }

    function enter(e: PointerEvent): void {
      if (e.pointerType !== 'mouse') return;
      engaged = false;
      wait();
    }

    function move(e: PointerEvent): void {
      if (e.pointerType === 'mouse' && !engaged) wait();
    }

    function leave(): void {
      clearTimeout(rest);
      engaged = false;
      finish();
    }

    function wheel(e: WheelEvent): void {
      if (!spec.enabled() || e.buttons !== 0) return;
      if (!engaged) {
        wait();
        return;
      }
      const size = wheelSize(e);
      if (size === null) return;
      e.preventDefault();
      clearTimeout(idle);
      idle = setTimeout(finish, WHEEL_SETTLE_MS);
      const steps = stepper.steps(e);
      if (steps === 0) return;
      adjusting = true;
      spec.nudge(steps, size);
    }

    const listening = new AbortController();
    const options = { signal: listening.signal };
    element.addEventListener('pointerenter', enter, options);
    element.addEventListener('pointermove', move, options);
    element.addEventListener('pointerleave', leave, options);
    element.addEventListener('wheel', wheel, { ...options, passive: false });

    return () => {
      listening.abort();
      clearTimeout(rest);
      finish();
    };
  };
}
