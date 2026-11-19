import type { Point } from '#lib/gestures/point.ts';
import { travelOf, type TrackedPointer } from '#lib/gestures/pointers.ts';
import {
  movedPointer,
  recognizerTraits,
  type Recognizer,
  type RecognizerHost,
  type RecognizerOptions
} from '#lib/gestures/recognizer.ts';
import { thresholds } from '#lib/gestures/thresholds.ts';

export type LongPressSpec = RecognizerOptions & {
  accepts?: (point: Point) => boolean;
  start?: (point: Point) => void;
  move?: (point: Point) => void;
  end?: (point: Point) => void;
  cancel?: () => void;
};

type Hold = {
  id: number;
  point: Point;
  active: boolean;
  timer: ReturnType<typeof setTimeout> | null;
};

export function longPressRecognizer(spec: LongPressSpec): Recognizer {
  const limits = thresholds(spec.thresholds);
  let hold: Hold | null = null;

  function held(pointer: TrackedPointer): Hold | null {
    return hold?.id === pointer.id ? hold : null;
  }

  function reset(): void {
    if (hold?.timer) clearTimeout(hold.timer);
    hold = null;
  }

  function fire(current: Hold, host: RecognizerHost): void {
    current.timer = null;
    if (hold !== current) return;
    if (spec.accepts?.(current.point) === false || !host.claim()) {
      reset();
      return;
    }
    current.active = true;
    spec.start?.(current.point);
  }

  function down(pointer: TrackedPointer, host: RecognizerHost): void {
    reset();
    const current: Hold = { id: pointer.id, point: pointer.current, active: false, timer: null };
    current.timer = setTimeout(() => fire(current, host), limits.longPressMs);
    hold = current;
  }

  function move(moved: readonly TrackedPointer[]): boolean {
    const current = hold;
    const pointer = movedPointer(moved, current?.id);
    if (!current || !pointer) return false;
    current.point = pointer.current;
    if (current.active) spec.move?.(current.point);
    else if (travelOf(pointer) > limits.tapSlop) reset();
    return false;
  }

  function up(pointer: TrackedPointer): void {
    const current = held(pointer);
    if (!current) return;
    reset();
    if (current.active) spec.end?.(pointer.current);
  }

  function cancel(): void {
    const active = hold?.active === true;
    reset();
    if (active) spec.cancel?.();
  }

  return {
    ...recognizerTraits(spec, { pointerCount: 1, exclusive: true, resumable: false }),
    pointerIds: () => (hold ? [hold.id] : []),
    down,
    move,
    up,
    cancel,
    reset
  };
}
