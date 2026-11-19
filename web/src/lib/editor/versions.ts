import type { CopyRecord } from '#lib/api/copies.ts';
import type { EditHistoryEntry } from '#lib/api/edits.ts';
import { copyIndex } from '#lib/browse/assetKey.ts';
import { currentEntry, entryText } from '#lib/editor/historySteps.ts';
import { countOf } from '#lib/utils/plural.ts';

export const ORIGINAL_NAME = 'Original';

export interface Version {
  id: string;
  name: string;
  copyIndex: number | null;
  named: boolean;
}

export function copyVersion(copy: CopyRecord): Version {
  const index = copyIndex(copy.id);
  return {
    id: copy.id,
    name: copy.name ?? `Copy ${index ?? copy.id.split('_').at(-1)}`,
    copyIndex: index,
    named: copy.name !== null
  };
}

export function versionsOf(master: string, copies: readonly CopyRecord[]): Version[] {
  const original: Version = { id: master, name: ORIGINAL_NAME, copyIndex: null, named: false };
  return [original, ...copies.map(copyVersion)];
}

export function lastAction(entries: readonly EditHistoryEntry[], isRaw: boolean): string | null {
  const current = currentEntry(entries);
  return current ? entryText(entries, current, isRaw) : null;
}

export function versionSummary(
  version: Version,
  entries: readonly EditHistoryEntry[] | undefined,
  isRaw: boolean
): string {
  const parts: string[] = [];
  if (version.copyIndex !== null && version.named) parts.push(`Copy ${version.copyIndex}`);
  if (entries) {
    const action = lastAction(entries, isRaw);
    if (entries.length === 0) parts.push('No edits');
    else parts.push(...(action ? [action] : []), countOf(entries.length, 'step'));
  }
  return parts.join(' · ');
}
