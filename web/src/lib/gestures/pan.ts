import { dominantAxis, lockedTo, type Axis } from '#lib/gestures/axis.ts';
import type { Point } from '#lib/gestures/point.ts';
import { offsetOf, travelOf, velocityOf, type TrackedPointer } from '#lib/gestures/pointers.ts';
import {
  movedPointer,
  recognizerTraits,
  type Recognizer,
  type RecognizerOptions
} from '#lib/gestures/recognizer.ts';
import { thresholds } from '#lib/gestures/thresholds.ts';

export type PanUpdate = {
  x: number;
  y: number;
  dx: number;
  dy: number;
  stepX: number;
  stepY: number;
  axis: Axis | null;
};

export type PanEnd = PanUpdate & { vx: number; vy: number };

export type PanSpec = RecognizerOptions & {
  lockAxis?: boolean;
  start?: (pan: PanUpdate) => void;
  move?: (pan: PanUpdate) => void;
  end?: (pan: PanEnd) => void;
  cancel?: () => void;
};

type Track = { id: number; active: boolean; axis: Axis | null; last: Point };

export function panRecognizer(spec: PanSpec): Recognizer {
  const limits = thresholds(spec.thresholds);
  let track: Track | null = null;

  function tracked(pointer: TrackedPointer): Track | null {
    return track?.id === pointer.id ? track : null;
  }

  function update(current: Track, pointer: TrackedPointer): PanUpdate {
    const delta = lockedTo(current.axis, offsetOf(pointer));
    const stepX = delta.x - current.last.x;
    const stepY = delta.y - current.last.y;
    current.last = delta;
    const { x, y } = pointer.current;
    return { x, y, dx: delta.x, dy: delta.y, stepX, stepY, axis: current.axis };
  }

  function down(pointer: TrackedPointer): void {
    track = { id: pointer.id, active: false, axis: null, last: { x: 0, y: 0 } };
  }

  function move(moved: readonly TrackedPointer[]): boolean {
    const current = track;
    const pointer = movedPointer(moved, current?.id);
    if (!current || !pointer) return false;
    if (current.active) {
      spec.move?.(update(current, pointer));
      return false;
    }
    if (travelOf(pointer) <= limits.panSlop) return false;
    current.active = true;
    current.axis = spec.lockAxis ? dominantAxis(offsetOf(pointer)) : null;
    spec.start?.(update(current, pointer));
    return true;
  }

  function up(pointer: TrackedPointer): void {
    const current = tracked(pointer);
    track = null;
    if (!current?.active) return;
    const velocity = lockedTo(current.axis, velocityOf(pointer));
    spec.end?.({ ...update(current, pointer), vx: velocity.x, vy: velocity.y });
  }

  function reset(): void {
    track = null;
  }

  function cancel(): void {
    const active = track?.active === true;
    reset();
    if (active) spec.cancel?.();
  }

  return {
    ...recognizerTraits(spec, { pointerCount: 1, exclusive: false, resumable: true }),
    pointerIds: () => (track ? [track.id] : []),
    down,
    move,
    up,
    cancel,
    reset
  };
}
