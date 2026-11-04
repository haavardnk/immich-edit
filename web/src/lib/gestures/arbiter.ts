import {
  pointerTable,
  type PointerKind,
  type PointerSample,
  type PointerTable,
  type TrackedPointer
} from '#lib/gestures/pointers.ts';
import type { Recognizer, RecognizerHost } from '#lib/gestures/recognizer.ts';
import { DEFAULT_THRESHOLDS } from '#lib/gestures/thresholds.ts';

export type ArbiterOptions = {
  pointerTypes?: readonly PointerKind[];
  velocityWindowMs?: number;
  onOwnerChange?: (owner: Recognizer | null) => void;
};

export type Arbiter = {
  readonly pointers: PointerTable;
  down: (sample: PointerSample) => void;
  move: (sample: PointerSample) => void;
  moveMany: (samples: readonly PointerSample[]) => void;
  up: (sample: PointerSample) => void;
  cancel: (pointerId: number) => void;
  reset: () => void;
  locked: () => Recognizer | null;
  contenders: () => readonly Recognizer[];
};

type Contender = { recognizer: Recognizer; host: RecognizerHost };

function tracked(pointer: TrackedPointer | null): pointer is TrackedPointer {
  return pointer !== null;
}

function holdsPointers(contender: Contender): boolean {
  return contender.recognizer.pointerIds().length > 0;
}

export function gestureArbiter(
  recognizers: readonly Recognizer[],
  options: ArbiterOptions = {}
): Arbiter {
  const pointers = pointerTable(options.velocityWindowMs ?? DEFAULT_THRESHOLDS.velocityWindowMs);
  const entries: Contender[] = recognizers.map((recognizer) => ({
    recognizer,
    host: { pointers, claim: () => claim(recognizer) }
  }));
  let candidates: Contender[] = [];
  let owner: Contender | null = null;
  let consumed = false;

  function setOwner(next: Contender | null): void {
    if (owner === next) return;
    owner = next;
    options.onOwnerChange?.(next?.recognizer ?? null);
  }

  function lock(winner: Contender): void {
    const losers = candidates.filter((candidate) => candidate !== winner);
    candidates = [];
    for (const loser of losers) loser.recognizer.reset();
    setOwner(winner);
  }

  function claim(recognizer: Recognizer): boolean {
    const winner = candidates.find((candidate) => candidate.recognizer === recognizer);
    if (owner || !winner) return false;
    lock(winner);
    return true;
  }

  function accepts(pointerType: string): boolean {
    return !options.pointerTypes || options.pointerTypes.some((kind) => kind === pointerType);
  }

  function compete(count: 1 | 2, pointer: TrackedPointer, resuming = false): void {
    candidates = entries.filter(
      ({ recognizer }) =>
        recognizer.pointerCount === count &&
        (!resuming || recognizer.resumable) &&
        recognizer.enabled()
    );
    for (const candidate of [...candidates]) {
      if (owner) return;
      candidate.recognizer.down(pointer, candidate.host);
    }
  }

  function dropCandidates(): void {
    const dropped = candidates;
    candidates = [];
    for (const candidate of dropped) candidate.recognizer.reset();
  }

  function regroup(): void {
    if (consumed || owner || candidates.length > 0) return;
    const first = pointers.first();
    if (!first) return;
    const count = pointers.count();
    if (count === 2) compete(2, first);
    if (count !== 1) return;
    const rebased = pointers.rebase(first.id);
    if (rebased) compete(1, rebased, true);
  }

  function down(sample: PointerSample): void {
    if (!accepts(sample.pointerType)) return;
    if (pointers.has(sample.pointerId)) cancel(sample.pointerId);
    const pointer = pointers.down(sample);
    const count = pointers.count();
    if (count === 1) {
      consumed = false;
      setOwner(null);
      compete(1, pointer);
      return;
    }
    if (count !== 2 || consumed) return;
    const displaced = owner;
    setOwner(null);
    displaced?.recognizer.cancel();
    dropCandidates();
    compete(2, pointer);
  }

  function moveMany(samples: readonly PointerSample[]): void {
    const moved = [...new Set(samples.map((sample) => pointers.move(sample)).filter(tracked))];
    if (moved.length === 0) return;
    if (owner) {
      owner.recognizer.move(moved, owner.host);
      return;
    }
    for (const candidate of candidates) {
      if (candidate.recognizer.move(moved, candidate.host)) lock(candidate);
      if (owner) return;
    }
  }

  function up(sample: PointerSample): void {
    const pointer = pointers.up(sample);
    if (!pointer) return;
    const current = owner;
    if (current) {
      current.recognizer.up(pointer, current.host);
      if (!holdsPointers(current)) setOwner(null);
    } else {
      const ending = candidates;
      for (const candidate of ending) candidate.recognizer.up(pointer, candidate.host);
      candidates = ending.filter(holdsPointers);
    }
    regroup();
  }

  function cancel(pointerId: number): void {
    if (!pointers.cancel(pointerId)) return;
    consumed = true;
    const current = owner;
    setOwner(null);
    current?.recognizer.cancel();
    dropCandidates();
  }

  function reset(): void {
    const current = owner;
    candidates = [];
    consumed = false;
    pointers.clear();
    setOwner(null);
    current?.recognizer.cancel();
    for (const entry of entries) entry.recognizer.reset();
  }

  return {
    pointers,
    down,
    move: (sample) => moveMany([sample]),
    moveMany,
    up,
    cancel,
    reset,
    locked: () => owner?.recognizer ?? null,
    contenders: () =>
      owner ? [owner.recognizer] : candidates.map((candidate) => candidate.recognizer)
  };
}
