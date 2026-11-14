import type { TagSummary } from '#lib/api/tags.ts';
import { isManagedTag, toTagRef } from '#lib/browse/managedTags.ts';
import { formatChord, parseChord } from '#lib/shortcuts/chord.ts';
import { KEYBINDS, type Keybind } from '#lib/shortcuts/table.ts';
import type { TagRef } from '#lib/types/asset.ts';
import { isMac } from '#lib/utils/platform.ts';

export const MAX_RECENT_TAGS = 8;

const RECENT_TAG_KEYS =
  (KEYBINDS as readonly Keybind[]).find((bind) => bind.id === 'recentTag')?.keys ?? [];
const RECENT_TAG_CODES = RECENT_TAG_KEYS.map((spec) => parseChord(spec).key);

export function withRecentTag(list: readonly TagRef[], tag: TagRef): TagRef[] {
  if (isManagedTag(tag)) return [...list];
  return [tag, ...list.filter((entry) => entry.id !== tag.id)].slice(0, MAX_RECENT_TAGS);
}

export function liveRecentTags(list: readonly TagRef[], live: readonly TagSummary[]): TagRef[] {
  const byId = new Map(live.map((tag) => [tag.id, tag]));
  return list.flatMap((entry) => {
    const current = byId.get(entry.id);
    return current ? [toTagRef(current)] : [];
  });
}

export function storedRecentTags(value: unknown): TagRef[] {
  if (!Array.isArray(value)) return [];
  return value
    .filter(
      (entry): entry is TagRef =>
        typeof entry === 'object' &&
        entry !== null &&
        typeof (entry as Partial<TagRef>).id === 'string' &&
        typeof (entry as Partial<TagRef>).value === 'string'
    )
    .map((entry) => ({
      id: entry.id,
      name: typeof entry.name === 'string' ? entry.name : entry.value,
      value: entry.value
    }))
    .slice(0, MAX_RECENT_TAGS);
}

export function sameRecentTags(a: readonly TagRef[], b: readonly TagRef[]): boolean {
  return (
    a.length === b.length &&
    a.every((tag, index) => tag.id === b[index]?.id && tag.value === b[index]?.value)
  );
}

export function recentTagSlot(code: string): number | null {
  const slot = RECENT_TAG_CODES.indexOf(code);
  return slot < 0 ? null : slot;
}

export function recentTagChord(slot: number, mac: boolean = isMac): string | null {
  const spec = RECENT_TAG_KEYS[slot];
  return spec ? formatChord(spec, mac) : null;
}
