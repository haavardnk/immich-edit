import type { CurveChannel } from '#lib/types/edits.ts';

export class CurveEditorStore {
  channel = $state<CurveChannel>('composite');
  selected = $state<number | null>(null);
  poppedOut = $state(false);
}

export const curveEditor = new CurveEditorStore();
