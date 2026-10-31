import { countOf } from '#lib/utils/plural.ts';
import { isCopy, sourceId } from './assetKey';
import { ownsAsset } from './ownership';

export interface TrashTarget {
  id: string;
  ownerId?: string;
}

export interface TrashPlan {
  masters: string[];
  copies: string[];
  versions: string[];
  skipped: number;
}

export interface TrashDialogText {
  title: string;
  confirm: string;
  lines: string[];
}

export const TRASH_CHUNK = 1000;

export const NOT_OWNER_TRASH =
  'Shared by another Immich user. Only the owner can move a photo to the trash.';
export const TRASH_OFF =
  "The Immich trash is turned off, so Immich would delete photos for good. Turn the trash on in Immich's administration settings first.";

export function planTrash(
  picked: readonly TrashTarget[],
  loaded: readonly { id: string; copyOf?: string }[]
): TrashPlan {
  const masters = new Set<string>();
  const pickedCopies = new Set<string>();
  let skipped = 0;
  for (const target of picked) {
    if (isCopy(target.id)) pickedCopies.add(target.id);
    else if (ownsAsset(target)) masters.add(target.id);
    else skipped += 1;
  }
  const copies = [...pickedCopies].filter((id) => !masters.has(sourceId(id)));
  const versions = loaded
    .filter((a) => a.copyOf !== undefined && masters.has(a.copyOf))
    .map((a) => a.id);
  return { masters: [...masters], copies, versions, skipped };
}

export function trashBlocked(plan: TrashPlan, trashOn: boolean): string | null {
  if (plan.masters.length === 0 && plan.copies.length === 0) return NOT_OWNER_TRASH;
  if (plan.masters.length > 0 && !trashOn) return TRASH_OFF;
  return null;
}

export function chunked<T>(items: readonly T[], size = TRASH_CHUNK): T[][] {
  const out: T[][] = [];
  for (let i = 0; i < items.length; i += size) out.push(items.slice(i, i + size));
  return out;
}

export function trashLabel(copy: boolean): string {
  return copy ? 'Delete virtual copy' : 'Move to Immich trash';
}

export function trashPeriod(trashDays: number | null): string {
  return trashDays === null ? 'when its trash period ends' : `after ${countOf(trashDays, 'day')}`;
}

export function trashDialogText(plan: TrashPlan, trashDays: number | null): TrashDialogText {
  const masters = plan.masters.length;
  const lines: string[] = [];
  if (masters > 0) {
    const them = masters === 1 ? 'it' : 'them';
    lines.push(
      `${countOf(masters, 'photo moves', 'photos move')} to the Immich trash. Immich deletes ${them} for good ${trashPeriod(trashDays)}; restore ${them} from the Immich trash until then.`
    );
  }
  if (plan.versions.length > 0) {
    lines.push(
      `${countOf(plan.versions.length, 'virtual copy goes', 'virtual copies go')} with ${masters === 1 ? 'it' : 'them'} and ${plan.versions.length === 1 ? 'comes' : 'come'} back on restore.`
    );
  }
  if (plan.copies.length > 0) {
    lines.push(
      `${countOf(plan.copies.length, 'virtual copy is', 'virtual copies are')} deleted from immich-edit. This cannot be undone.`
    );
  }
  if (plan.skipped > 0) {
    lines.push(
      `${countOf(plan.skipped, 'photo shared', 'photos shared')} by another Immich user ${plan.skipped === 1 ? 'is' : 'are'} skipped.`
    );
  }
  if (masters > 0) {
    return { title: trashLabel(false), confirm: 'Move to trash', lines };
  }
  return {
    title: plan.copies.length === 1 ? 'Delete virtual copy' : 'Delete virtual copies',
    confirm: 'Delete',
    lines
  };
}

export function trashedMessage(trashed: number, deleted: number): string {
  const parts: string[] = [];
  if (trashed > 0) parts.push(`Moved ${countOf(trashed, 'photo', 'photos')} to the Immich trash`);
  if (deleted > 0) {
    const copies = countOf(deleted, 'virtual copy', 'virtual copies');
    parts.push(trashed > 0 ? `deleted ${copies}` : `Deleted ${copies}`);
  }
  return parts.join(' and ');
}
