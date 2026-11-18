import type { Point } from '#lib/gestures/point.ts';
import { distance, travelOf, type TrackedPointer } from '#lib/gestures/pointers.ts';
import {
  movedPointer,
  recognizerTraits,
  type Recognizer,
  type RecognizerOptions
} from '#lib/gestures/recognizer.ts';
import { thresholds } from '#lib/gestures/thresholds.ts';

export type TapSpec = RecognizerOptions & {
  tap?: (point: Point) => void;
  doubleTap?: (point: Point) => void;
};

type Press = { id: number; moved: boolean; second: boolean };

type PendingTap = { point: Point; time: number; timer: ReturnType<typeof setTimeout> };

export function tapRecognizer(spec: TapSpec): Recognizer {
  const limits = thresholds(spec.thresholds);
  let press: Press | null = null;
  let pending: PendingTap | null = null;

  function takePending(): PendingTap | null {
    const current = pending;
    if (current) clearTimeout(current.timer);
    pending = null;
    return current;
  }

  function confirmPending(): void {
    const current = takePending();
    if (current) spec.tap?.(current.point);
  }

  function pairsWithPending(pointer: TrackedPointer): boolean {
    return (
      pending !== null &&
      pointer.startTime - pending.time <= limits.doubleTapMs &&
      distance(pointer.start, pending.point) <= limits.doubleTapSlop
    );
  }

  function down(pointer: TrackedPointer): void {
    const second = pairsWithPending(pointer);
    if (second) takePending();
    else confirmPending();
    press = { id: pointer.id, moved: false, second };
  }

  function move(moved: readonly TrackedPointer[]): boolean {
    const pointer = movedPointer(moved, press?.id);
    if (press && pointer && travelOf(pointer) > limits.tapSlop) press.moved = true;
    return false;
  }

  function up(pointer: TrackedPointer): void {
    const current = press;
    press = null;
    if (!current || current.id !== pointer.id) return;
    if (current.moved || travelOf(pointer) > limits.tapSlop) return;
    if (current.second) {
      spec.doubleTap?.(pointer.start);
      return;
    }
    if (!spec.doubleTap) {
      spec.tap?.(pointer.start);
      return;
    }
    pending = {
      point: pointer.start,
      time: pointer.time,
      timer: setTimeout(confirmPending, limits.doubleTapMs)
    };
  }

  function reset(): void {
    press = null;
    takePending();
  }

  return {
    ...recognizerTraits(spec, { pointerCount: 1, exclusive: false, resumable: false }),
    pointerIds: () => (press ? [press.id] : []),
    down,
    move,
    up,
    cancel: reset,
    reset
  };
}
