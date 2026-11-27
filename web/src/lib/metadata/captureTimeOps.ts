import { writeCaptureTimes } from '#lib/api/assets.ts';
import { withExif } from '#lib/browse/assetMeta.ts';
import { sourceId } from '#lib/browse/assetKey.ts';
import {
  planWrites,
  wallOf,
  type CapturePlan,
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

interface Saved {
  count: number;
  failed: number;
  undo: () => Promise<void>;
}

async function saveTimes(
  plan: CapturePlan,
  changes: readonly ClockChange[],
  assets: ReadonlyMap<string, AssetSummary>
): Promise<Saved | null> {
  let failed: Set<string>;
  try {
    failed = await writeChunks(plan.write);
  } catch (error) {
    toasts.fail('Change capture time', error);
    return null;
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
  return {
    count: written.size,
    failed: failed.size,
    undo: () => undoTimes(undo, restored, assets)
  };
}

export async function applyCaptureTimes(
  changes: readonly ClockChange[],
  assets: ReadonlyMap<string, AssetSummary>
): Promise<boolean> {
  const plan = planWrites(changes);
  if (plan.write.length === 0) return true;
  if (!(await metadataConsent.gate())) return false;
  const saved = await saveTimes(plan, changes, assets);
  if (!saved) return false;
  toasts.push('success', `Changed the capture time of ${countOf(saved.count, 'photo')}`, {
    description:
      saved.failed > 0 ? `${countOf(saved.failed, 'photo')} could not be changed` : undefined,
    action: { label: 'Undo', run: () => void saved.undo() }
  });
  return true;
}

export async function captureTimesAfter(
  changes: readonly ClockChange[],
  assets: ReadonlyMap<string, AssetSummary>
): Promise<FollowUp | null> {
  const plan = planWrites(changes);
  if (plan.write.length === 0) return null;
  const saved = await saveTimes(plan, changes, assets);
  if (!saved) return null;
  if (saved.failed > 0) {
    toasts.push('warn', `Could not change the capture time of ${countOf(saved.failed, 'photo')}`);
  }
  if (saved.count === 0) return null;
  return {
    note: `and changed the capture time of ${countOf(saved.count, 'photo')}`,
    undo: saved.undo
  };
}
