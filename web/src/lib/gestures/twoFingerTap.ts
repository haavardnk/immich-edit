import type { Point } from '#lib/gestures/point.ts';
import { centroid, travelOf, type TrackedPointer } from '#lib/gestures/pointers.ts';
import {
  recognizerTraits,
  type Recognizer,
  type RecognizerHost,
  type RecognizerOptions
} from '#lib/gestures/recognizer.ts';
import { thresholds } from '#lib/gestures/thresholds.ts';

export type TwoFingerTapSpec = RecognizerOptions & {
  tap?: (point: Point) => void;
};

type Pair = {
  ids: readonly [number, number];
  point: Point;
  startTime: number;
  lifted: number[];
  moved: boolean;
};

export function twoFingerTapRecognizer(spec: TwoFingerTapSpec): Recognizer {
  const limits = thresholds(spec.thresholds);
  let pair: Pair | null = null;

  function member(pointer: TrackedPointer): Pair | null {
    return pair?.ids.includes(pointer.id) ? pair : null;
  }

  function strayed(pointer: TrackedPointer): boolean {
    return travelOf(pointer) > limits.tapSlop;
  }

  function down(_pointer: TrackedPointer, host: RecognizerHost): void {
    const a = host.pointers.first();
    const b = host.pointers.second();
    if (!a || !b) return;
    pair = {
      ids: [a.id, b.id],
      point: centroid(a.start, b.start),
      startTime: Math.min(a.startTime, b.startTime),
      lifted: [],
      moved: false
    };
  }

  function move(moved: readonly TrackedPointer[]): boolean {
    const current = pair;
    if (current && moved.some((pointer) => member(pointer) && strayed(pointer))) {
      current.moved = true;
    }
    return false;
  }

  function up(pointer: TrackedPointer): void {
    const current = member(pointer);
    if (!current) return;
    if (strayed(pointer)) current.moved = true;
    current.lifted.push(pointer.id);
    if (current.lifted.length < current.ids.length) return;
    pair = null;
    if (!current.moved && pointer.time - current.startTime <= limits.longPressMs) {
      spec.tap?.(current.point);
    }
  }

  function reset(): void {
    pair = null;
  }

  return {
    ...recognizerTraits(spec, { pointerCount: 2, exclusive: false, resumable: false }),
    pointerIds: () =>
      pair && !pair.moved ? pair.ids.filter((id) => !pair?.lifted.includes(id)) : [],
    down,
    move,
    up,
    cancel: reset,
    reset
  };
}
