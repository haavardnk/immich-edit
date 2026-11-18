import { gestureArbiter, type ArbiterOptions } from '#lib/gestures/arbiter.ts';
import type { PointerSample } from '#lib/gestures/pointers.ts';
import type { Recognizer } from '#lib/gestures/recognizer.ts';
import type { Attachment } from 'svelte/attachments';

export type GesturesSpec = Omit<ArbiterOptions, 'onOwnerChange'> & {
  recognizers: readonly Recognizer[];
  touchAction?: string | null;
};

function sampleOf(e: PointerEvent): PointerSample {
  return {
    pointerId: e.pointerId,
    pointerType: e.pointerType,
    clientX: e.clientX,
    clientY: e.clientY,
    timeStamp: e.timeStamp
  };
}

export function gestures(spec: GesturesSpec): Attachment<HTMLElement> {
  return (element) => {
    const captured = new Set<number>();
    const pending = new Map<number, PointerSample>();
    const arbiter = gestureArbiter(spec.recognizers, { ...spec, onOwnerChange: capture });
    const touchAction = spec.touchAction === undefined ? 'none' : spec.touchAction;
    const previousTouchAction = element.style.touchAction;
    let frame: number | null = null;

    function release(): void {
      for (const pointerId of captured) {
        if (element.hasPointerCapture?.(pointerId)) element.releasePointerCapture(pointerId);
      }
      captured.clear();
    }

    function capture(owner: Recognizer | null): void {
      release();
      for (const pointerId of owner?.pointerIds() ?? []) {
        if (!arbiter.pointers.has(pointerId)) continue;
        element.setPointerCapture?.(pointerId);
        captured.add(pointerId);
      }
    }

    function flush(): void {
      frame = null;
      const samples = [...pending.values()];
      pending.clear();
      arbiter.moveMany(samples);
    }

    function settle(): void {
      if (frame !== null) cancelAnimationFrame(frame);
      flush();
    }

    function holdsTouch(): boolean {
      const owner = arbiter.locked();
      if (owner) return owner.exclusive;
      return arbiter
        .contenders()
        .some((recognizer) => recognizer.exclusive && recognizer.pointerCount > 1);
    }

    function pressedByTouch(): boolean {
      return arbiter.pointers.list().some((pointer) => pointer.type === 'touch');
    }

    function cancel(e: PointerEvent): void {
      if (!arbiter.pointers.has(e.pointerId)) return;
      settle();
      arbiter.cancel(e.pointerId);
    }

    function down(e: PointerEvent): void {
      if (e.button !== 0) return;
      settle();
      arbiter.down(sampleOf(e));
    }

    function move(e: PointerEvent): void {
      if (!arbiter.pointers.has(e.pointerId)) return;
      if (e.buttons === 0) {
        cancel(e);
        return;
      }
      pending.set(e.pointerId, sampleOf(e));
      if (frame === null) frame = requestAnimationFrame(flush);
    }

    function up(e: PointerEvent): void {
      if (!arbiter.pointers.has(e.pointerId)) return;
      settle();
      captured.delete(e.pointerId);
      arbiter.up(sampleOf(e));
    }

    function lostCapture(e: PointerEvent): void {
      if (e.target !== element || !captured.delete(e.pointerId)) return;
      cancel(e);
    }

    function touchMove(e: TouchEvent): void {
      if (e.cancelable && holdsTouch()) e.preventDefault();
    }

    function contextMenu(e: MouseEvent): void {
      if (!pressedByTouch()) return;
      const holding = arbiter
        .contenders()
        .some((recognizer) => recognizer.exclusive && recognizer.pointerCount === 1);
      if (holding) e.preventDefault();
    }

    const listening = new AbortController();
    const options = { signal: listening.signal };
    if (touchAction !== null) element.style.touchAction = touchAction;
    element.addEventListener('pointerdown', down, options);
    element.addEventListener('pointermove', move, options);
    element.addEventListener('pointerup', up, options);
    element.addEventListener('pointercancel', cancel, options);
    element.addEventListener('lostpointercapture', lostCapture, options);
    element.addEventListener('touchmove', touchMove, { ...options, passive: false });
    element.addEventListener('contextmenu', contextMenu, options);

    return () => {
      listening.abort();
      if (frame !== null) cancelAnimationFrame(frame);
      frame = null;
      pending.clear();
      arbiter.reset();
      release();
      if (touchAction !== null) element.style.touchAction = previousTouchAction;
    };
  };
}
