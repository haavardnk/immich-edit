import type { EditedAssetEntry } from '#lib/api/edits.ts';

const RECENT_EDITS = 4;

export function recentEdits(
  entries: readonly EditedAssetEntry[],
  limit = RECENT_EDITS
): EditedAssetEntry[] {
  return [...entries]
    .sort((a, b) => Date.parse(b.updated_at) - Date.parse(a.updated_at))
    .slice(0, limit);
}
