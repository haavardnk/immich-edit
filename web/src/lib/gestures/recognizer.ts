import type { PointerTable, TrackedPointer } from '#lib/gestures/pointers.ts';
import type { Thresholds } from '#lib/gestures/thresholds.ts';

export type RecognizerHost = {
  pointers: PointerTable;
  claim: () => boolean;
};

export type Recognizer = {
  readonly pointerCount: 1 | 2;
  readonly exclusive: boolean;
  readonly resumable: boolean;
  enabled: () => boolean;
  pointerIds: () => readonly number[];
  down: (pointer: TrackedPointer, host: RecognizerHost) => void;
  move: (moved: readonly TrackedPointer[], host: RecognizerHost) => boolean;
  up: (pointer: TrackedPointer, host: RecognizerHost) => void;
  cancel: () => void;
  reset: () => void;
};

export type RecognizerOptions = {
  enabled?: () => boolean;
  exclusive?: boolean;
  thresholds?: Partial<Thresholds>;
};

export type RecognizerTraits = Pick<
  Recognizer,
  'pointerCount' | 'exclusive' | 'resumable' | 'enabled'
>;

const always = (): boolean => true;

export function recognizerTraits(
  options: RecognizerOptions,
  defaults: Pick<Recognizer, 'pointerCount' | 'exclusive' | 'resumable'>
): RecognizerTraits {
  return {
    ...defaults,
    exclusive: options.exclusive ?? defaults.exclusive,
    enabled: options.enabled ?? always
  };
}

export function movedPointer(
  moved: readonly TrackedPointer[],
  pointerId: number | undefined
): TrackedPointer | null {
  return moved.find((pointer) => pointer.id === pointerId) ?? null;
}
