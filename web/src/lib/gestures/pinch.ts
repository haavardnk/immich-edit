import type { Point } from '#lib/gestures/point.ts';
import { centroid, distance, type TrackedPointer } from '#lib/gestures/pointers.ts';
import {
  recognizerTraits,
  type Recognizer,
  type RecognizerHost,
  type RecognizerOptions
} from '#lib/gestures/recognizer.ts';
import { thresholds } from '#lib/gestures/thresholds.ts';

export type PinchUpdate = { scale: number; x: number; y: number; dx: number; dy: number };

export type PinchSpec = RecognizerOptions & {
  start?: (pinch: PinchUpdate) => void;
  move?: (pinch: PinchUpdate) => void;
  end?: (pinch: PinchUpdate) => void;
  cancel?: () => void;
};

type Pair = {
  ids: readonly [number, number];
  span: number;
  origin: Point;
  active: boolean;
  last: PinchUpdate;
};

export function pinchRecognizer(spec: PinchSpec): Recognizer {
  const limits = thresholds(spec.thresholds);
  let pair: Pair | null = null;

  function member(pointer: TrackedPointer): Pair | null {
    return pair?.ids.includes(pointer.id) ? pair : null;
  }

  function touches(moved: readonly TrackedPointer[]): Pair | null {
    return moved.some((pointer) => member(pointer)) ? pair : null;
  }

  function measure(current: Pair, host: RecognizerHost): PinchUpdate | null {
    const a = host.pointers.get(current.ids[0]);
    const b = host.pointers.get(current.ids[1]);
    if (!a || !b) return null;
    const focal = centroid(a.current, b.current);
    return {
      scale: distance(a.current, b.current) / current.span,
      x: focal.x,
      y: focal.y,
      dx: focal.x - current.origin.x,
      dy: focal.y - current.origin.y
    };
  }

  function crossed(update: PinchUpdate): boolean {
    return (
      Math.abs(update.scale - 1) > limits.pinchSlop ||
      Math.hypot(update.dx, update.dy) > limits.panSlop
    );
  }

  function down(_pointer: TrackedPointer, host: RecognizerHost): void {
    const a = host.pointers.first();
    const b = host.pointers.second();
    if (!a || !b) return;
    const origin = centroid(a.current, b.current);
    pair = {
      ids: [a.id, b.id],
      span: Math.max(1, distance(a.current, b.current)),
      origin,
      active: false,
      last: { scale: 1, x: origin.x, y: origin.y, dx: 0, dy: 0 }
    };
  }

  function move(moved: readonly TrackedPointer[], host: RecognizerHost): boolean {
    const current = touches(moved);
    if (!current) return false;
    const update = measure(current, host);
    if (!update) return false;
    current.last = update;
    if (current.active) {
      spec.move?.(update);
      return false;
    }
    if (!crossed(update)) return false;
    current.active = true;
    spec.start?.(update);
    return true;
  }

  function up(pointer: TrackedPointer): void {
    const current = member(pointer);
    if (!current) return;
    pair = null;
    if (current.active) spec.end?.(current.last);
  }

  function reset(): void {
    pair = null;
  }

  function cancel(): void {
    const active = pair?.active === true;
    reset();
    if (active) spec.cancel?.();
  }

  return {
    ...recognizerTraits(spec, { pointerCount: 2, exclusive: true, resumable: false }),
    pointerIds: () => pair?.ids ?? [],
    down,
    move,
    up,
    cancel,
    reset
  };
}
