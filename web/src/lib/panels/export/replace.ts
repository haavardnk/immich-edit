import { trashPeriod } from '$lib/browse/trash';

export const REPLACE_NOT_OWNER =
  'Shared by another Immich user. Only the owner can replace the original.';
export const REPLACE_NONE_OWNED =
  'The selected photos are shared by another Immich user. Only the owner can replace the original.';
export const REPLACE_TWO_VERSIONS =
  'Two versions of one photo are selected. Replace original takes one version of each photo.';
export const REPLACE_TRASH_OFF =
  'The Immich trash is turned off, so Immich would delete the original for good.';
export const REPLACE_STACKS = "Replace original moves the upload into the original's stack.";

export function replaceBlocked({
  owned,
  trashOn,
  versionsOfOnePhoto = false,
  single = true
}: {
  owned: boolean;
  trashOn: boolean;
  versionsOfOnePhoto?: boolean;
  single?: boolean;
}): string | null {
  if (!owned) return single ? REPLACE_NOT_OWNER : REPLACE_NONE_OWNED;
  if (versionsOfOnePhoto) return REPLACE_TWO_VERSIONS;
  if (!trashOn) return REPLACE_TRASH_OFF;
  return null;
}

export function replaceWarning(trashDays: number | null): string {
  return `After the upload, the original goes to the Immich trash and is deleted for good ${trashPeriod(trashDays)}. A RAW original then survives only as this export.`;
}
