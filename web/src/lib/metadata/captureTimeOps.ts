import { writeCaptureTimes } from '#lib/api/assets.ts';
import { withExif } from '#lib/browse/assetMeta.ts';
import { sourceId } from '#lib/browse/assetKey.ts';
import type { CaptureShift, ShiftPlan } from '#lib/metadata/captureShift.ts';
import {
  planWrites,
  wallOf,
  type CaptureWrite,
  type ClockChange,
  type ClockTarget
} from '#lib/metadata/captureTimes.ts';
import type { FollowUp } from '#lib/browse/locationOps.ts';
import { immichZone } from '#lib/metadata/photoTime.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { libraryChanged } from '#lib/stores/libraryChange.ts';
import { metadataConsent } from '#lib/stores/metadataConsent.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import { countOf } from '#lib/utils/plural.ts';

const CHUNK = 100;

function only(shifts: readonly CaptureShift[], ids: ReadonlySet<string>): CaptureShift[] {
  return shifts.filter((shift) => ids.has(sourceId(shift.id)));
}

async function undoShifts(shifts: readonly CaptureShift[]): Promise<void> {
  if (shifts.length === 0) return;
  try {
    await writeChunks(shifts);
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
    failed = await writeChunks(write);
  } catch (error) {
    toasts.fail('Change capture time', error);
    return null;
  }
  libraryChanged();
  if (failed.size > 0) {
    toasts.push('warn', `Could not change the capture time of ${countOf(failed.size, 'photo')}`);
  }
  const fixed = new Set([...chosen].filter((id) => !failed.has(id)));
  if (fixed.size === 0) return null;
  const back = only(plan.undo, fixed);
  return { note: 'and changed their capture time', undo: () => undoShifts(back) };
}

async function writeChunks(items: readonly CaptureWrite[]): Promise<Set<string>> {
  const failed = new Set<string>();
  let error: unknown = null;
  for (let start = 0; start < items.length; start += CHUNK) {
    const chunk = items.slice(start, start + CHUNK);
    try {
      for (const id of (await writeCaptureTimes(chunk)).failed) failed.add(id);
    } catch (caught) {
      error ??= caught;
      for (const item of chunk) failed.add(sourceId(item.id));
    }
  }
  if (error !== null && failed.size === new Set(items.map((item) => sourceId(item.id))).size) {
    throw error;
  }
  return failed;
}

function patchTimes(
  assets: ReadonlyMap<string, AssetSummary>,
  targets: ReadonlyMap<string, ClockTarget>
): void {
  for (const [id, target] of targets) {
    const asset = assets.get(id);
    if (!asset) continue;
    browsing.patch(id, {
      localDateTime: new Date(wallOf(target)).toISOString(),
      exifInfo: withExif(asset.exifInfo, {
        dateTimeOriginal: new Date(target.instant).toISOString(),
        timeZone: immichZone(target.zone, target.instant)
      })
    });
  }
}

async function undoTimes(
  undo: readonly CaptureWrite[],
  restored: ReadonlyMap<string, ClockTarget>,
  assets: ReadonlyMap<string, AssetSummary>
): Promise<void> {
  if (undo.length === 0) return;
  let failed: Set<string>;
  try {
    failed = await writeChunks(undo);
  } catch (error) {
    toasts.fail('Undo capture time', error);
    return;
  }
  patchTimes(assets, new Map([...restored].filter(([id]) => !failed.has(sourceId(id)))));
  libraryChanged();
  if (failed.size > 0) {
    toasts.push('warn', `Could not undo the capture time of ${countOf(failed.size, 'photo')}`);
  }
}

export async function applyCaptureTimes(
  changes: readonly ClockChange[],
  assets: ReadonlyMap<string, AssetSummary>
): Promise<boolean> {
  const plan = planWrites(changes);
  if (plan.write.length === 0) return true;
  if (!(await metadataConsent.gate())) return false;
  let failed: Set<string>;
  try {
    failed = await writeChunks(plan.write);
  } catch (error) {
    toasts.fail('Change capture time', error);
    return false;
  }
  const sent = new Set(plan.write.map((item) => item.id));
  const written = new Map(
    changes
      .filter(({ photo }) => sent.has(photo.id) && !failed.has(sourceId(photo.id)))
      .map(({ photo, target }) => [photo.id, target])
  );
  patchTimes(assets, written);
  libraryChanged();
  const undo = plan.undo.filter((item) => written.has(item.id));
  const restored = new Map([...plan.restored].filter(([id]) => written.has(id)));
  toasts.push('success', `Changed the capture time of ${countOf(written.size, 'photo')}`, {
    description:
      failed.size > 0 ? `${countOf(failed.size, 'photo')} could not be changed` : undefined,
    action: { label: 'Undo', run: () => void undoTimes(undo, restored, assets) }
  });
  return true;
}
