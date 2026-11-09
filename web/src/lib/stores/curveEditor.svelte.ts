import type { CurveChannel } from '#lib/types/edits.ts';

export class CurveEditorStore {
  channel = $state<CurveChannel>('composite');
  poppedOut = $state(false);
}

export const curveEditor = new CurveEditorStore();
