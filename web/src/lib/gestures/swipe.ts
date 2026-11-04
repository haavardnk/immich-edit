import { across, along, dominantAxis, type Axis } from '#lib/gestures/axis.ts';
import { offsetOf, velocityOf, type TrackedPointer } from '#lib/gestures/pointers.ts';
import {
  movedPointer,
  recognizerTraits,
  type Recognizer,
  type RecognizerOptions
} from '#lib/gestures/recognizer.ts';
import { thresholds } from '#lib/gestures/thresholds.ts';

export type SwipeDirection = 'left' | 'right' | 'up' | 'down';

export type SwipeProgress = { dx: number; dy: number; axis: Axis };

export type SwipeEnd = {
  direction: SwipeDirection | null;
  dx: number;
  dy: number;
  velocity: number;
};

export type SwipeSpec = RecognizerOptions & {
  directions?: readonly SwipeDirection[];
  start?: (progress: SwipeProgress) => void;
  move?: (progress: SwipeProgress) => void;
  end?: (result: SwipeEnd) => void;
  cancel?: () => void;
};

type Track = { id: number; axis: Axis | null };

const ALL_DIRECTIONS: readonly SwipeDirection[] = ['left', 'right', 'up', 'down'];

function directionOf(axis: Axis, value: number): SwipeDirection | null {
  if (value === 0) return null;
  if (axis === 'x') return value < 0 ? 'left' : 'right';
  return value < 0 ? 'up' : 'down';
}

export function swipeRecognizer(spec: SwipeSpec): Recognizer {
  const limits = thresholds(spec.thresholds);
  const directions = spec.directions ?? ALL_DIRECTIONS;
  let track: Track | null = null;

  function allowed(direction: SwipeDirection | null): direction is SwipeDirection {
    return direction !== null && directions.includes(direction);
  }

  function tracked(pointer: TrackedPointer): Track | null {
    return track?.id === pointer.id ? track : null;
  }

  function progress(axis: Axis, pointer: TrackedPointer): SwipeProgress {
    const offset = offsetOf(pointer);
    return { dx: offset.x, dy: offset.y, axis };
  }

  function claimable(pointer: TrackedPointer): Axis | null {
    const offset = offsetOf(pointer);
    const axis = dominantAxis(offset);
    const main = Math.abs(along(axis, offset));
    const cross = Math.abs(across(axis, offset));
    if (main <= limits.panSlop || main < limits.swipeDominance * cross) return null;
    return allowed(directionOf(axis, along(axis, offset))) ? axis : null;
  }

  function settle(axis: Axis, pointer: TrackedPointer): SwipeEnd {
    const offset = offsetOf(pointer);
    const main = along(axis, offset);
    const velocity = along(axis, velocityOf(pointer));
    const direction = directionOf(axis, main);
    const flung = Math.abs(velocity) >= limits.flingVelocity;
    const forward = Math.sign(velocity) === Math.sign(main);
    const commits = flung
      ? forward && Math.abs(main) >= limits.swipeMin / 2
      : Math.abs(main) >= limits.swipeMin;
    return {
      direction: commits && allowed(direction) ? direction : null,
      dx: offset.x,
      dy: offset.y,
      velocity
    };
  }

  function down(pointer: TrackedPointer): void {
    track = { id: pointer.id, axis: null };
  }

  function move(moved: readonly TrackedPointer[]): boolean {
    const current = track;
    const pointer = movedPointer(moved, current?.id);
    if (!current || !pointer) return false;
    if (current.axis) {
      spec.move?.(progress(current.axis, pointer));
      return false;
    }
    const axis = claimable(pointer);
    if (!axis) return false;
    current.axis = axis;
    spec.start?.(progress(axis, pointer));
    return true;
  }

  function up(pointer: TrackedPointer): void {
    const current = tracked(pointer);
    track = null;
    if (current?.axis) spec.end?.(settle(current.axis, pointer));
  }

  function reset(): void {
    track = null;
  }

  function cancel(): void {
    const active = track?.axis != null;
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
