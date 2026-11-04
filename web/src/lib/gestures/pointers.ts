import type { Point } from '#lib/gestures/point.ts';
import { DEFAULT_THRESHOLDS } from '#lib/gestures/thresholds.ts';

export type PointerKind = 'mouse' | 'pen' | 'touch';

export type PointerSample = {
  pointerId: number;
  pointerType: string;
  clientX: number;
  clientY: number;
  timeStamp: number;
};

export type Velocity = { x: number; y: number };

export type TimedPoint = Point & { time: number };

export type TrackedPointer = {
  readonly id: number;
  readonly type: string;
  readonly start: Point;
  readonly startTime: number;
  readonly current: Point;
  readonly time: number;
  readonly samples: readonly TimedPoint[];
};

export type PointerTable = {
  down: (sample: PointerSample) => TrackedPointer;
  move: (sample: PointerSample) => TrackedPointer | null;
  up: (sample: PointerSample) => TrackedPointer | null;
  cancel: (pointerId: number) => TrackedPointer | null;
  rebase: (pointerId: number) => TrackedPointer | null;
  get: (pointerId: number) => TrackedPointer | null;
  has: (pointerId: number) => boolean;
  count: () => number;
  first: () => TrackedPointer | null;
  second: () => TrackedPointer | null;
  list: () => readonly TrackedPointer[];
  clear: () => void;
};

type Entry = {
  id: number;
  type: string;
  start: Point;
  startTime: number;
  current: Point;
  time: number;
  samples: TimedPoint[];
};

const STILL: Velocity = { x: 0, y: 0 };

function pointOf(sample: PointerSample): Point {
  return { x: sample.clientX, y: sample.clientY };
}

export function pointerTable(velocityWindowMs = DEFAULT_THRESHOLDS.velocityWindowMs): PointerTable {
  const entries = new Map<number, Entry>();

  function record(entry: Entry, sample: PointerSample): Entry {
    const point = pointOf(sample);
    entry.current = point;
    entry.time = sample.timeStamp;
    entry.samples.push({ ...point, time: sample.timeStamp });
    const horizon = sample.timeStamp - velocityWindowMs;
    const fresh = entry.samples.findIndex((timed) => timed.time >= horizon);
    if (fresh > 0) entry.samples.splice(0, fresh);
    return entry;
  }

  function down(sample: PointerSample): TrackedPointer {
    const point = pointOf(sample);
    const entry: Entry = {
      id: sample.pointerId,
      type: sample.pointerType,
      start: point,
      startTime: sample.timeStamp,
      current: point,
      time: sample.timeStamp,
      samples: [{ ...point, time: sample.timeStamp }]
    };
    entries.delete(sample.pointerId);
    entries.set(sample.pointerId, entry);
    return entry;
  }

  function move(sample: PointerSample): TrackedPointer | null {
    const entry = entries.get(sample.pointerId);
    return entry ? record(entry, sample) : null;
  }

  function up(sample: PointerSample): TrackedPointer | null {
    const entry = entries.get(sample.pointerId);
    if (!entry) return null;
    entries.delete(sample.pointerId);
    return record(entry, sample);
  }

  function cancel(pointerId: number): TrackedPointer | null {
    const entry = entries.get(pointerId);
    if (!entry) return null;
    entries.delete(pointerId);
    return entry;
  }

  function rebase(pointerId: number): TrackedPointer | null {
    const entry = entries.get(pointerId);
    if (!entry) return null;
    entry.start = entry.current;
    entry.startTime = entry.time;
    entry.samples = [{ ...entry.current, time: entry.time }];
    return entry;
  }

  function nth(index: number): TrackedPointer | null {
    return [...entries.values()][index] ?? null;
  }

  return {
    down,
    move,
    up,
    cancel,
    rebase,
    get: (pointerId) => entries.get(pointerId) ?? null,
    has: (pointerId) => entries.has(pointerId),
    count: () => entries.size,
    first: () => nth(0),
    second: () => nth(1),
    list: () => [...entries.values()],
    clear: () => entries.clear()
  };
}

export function velocityOf(pointer: TrackedPointer): Velocity {
  const first = pointer.samples[0];
  const last = pointer.samples.at(-1);
  if (!first || !last) return STILL;
  const elapsed = last.time - first.time;
  if (elapsed <= 0) return STILL;
  return { x: (last.x - first.x) / elapsed, y: (last.y - first.y) / elapsed };
}

export function offsetOf(pointer: TrackedPointer): Point {
  return { x: pointer.current.x - pointer.start.x, y: pointer.current.y - pointer.start.y };
}

export function travelOf(pointer: TrackedPointer): number {
  const offset = offsetOf(pointer);
  return Math.hypot(offset.x, offset.y);
}

export function distance(a: Point, b: Point): number {
  return Math.hypot(b.x - a.x, b.y - a.y);
}

export function centroid(a: Point, b: Point): Point {
  return { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 };
}
