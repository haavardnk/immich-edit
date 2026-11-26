import { shiftCaptureTimes } from '#lib/api/assets.ts';
import { sourceId } from '#lib/browse/assetKey.ts';
import type { CaptureShift, ShiftPlan } from '#lib/metadata/captureShift.ts';
import type { FollowUp } from '#lib/browse/locationOps.ts';
import { libraryChanged } from '#lib/stores/libraryChange.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { countOf } from '#lib/utils/plural.ts';

function only(shifts: readonly CaptureShift[], ids: ReadonlySet<string>): CaptureShift[] {
  return shifts.filter((shift) => ids.has(sourceId(shift.id)));
}

async function undoShifts(shifts: readonly CaptureShift[]): Promise<void> {
  if (shifts.length === 0) return;
  try {
    await shiftCaptureTimes(shifts);
  } catch (error) {
    toasts.fail('Undo capture time', error);
    return;
  }
  libraryChanged();
}

export async function correctTimes(
  plan: ShiftPlan,
  ids: readonly string[]
): Promise<FollowUp | null> {
  const chosen = new Set(ids.map(sourceId));
  const write = only(plan.write, chosen);
  if (write.length === 0) return null;
  let failed: Set<string>;
  try {
    failed = new Set((await shiftCaptureTimes(write)).failed);
  } catch (error) {
    toasts.fail('Correct capture time', error);
    return null;
  }
  libraryChanged();
  if (failed.size > 0) {
    toasts.push('warn', `Could not correct the capture time of ${countOf(failed.size, 'photo')}`);
  }
  const fixed = new Set([...chosen].filter((id) => !failed.has(id)));
  if (fixed.size === 0) return null;
  const back = only(plan.undo, fixed);
  return { note: 'and corrected their capture time', undo: () => undoShifts(back) };
}
