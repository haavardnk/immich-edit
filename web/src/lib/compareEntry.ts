import { MAX_PANES, type CompareMode } from '$lib/stores/compare.svelte';

export type MultiMode = Exclude<CompareMode, 'single'>;

const SURVEY_FALLBACK = 6;

export function multiMembers(
  mode: MultiMode,
  orderedIds: string[],
  selected: Set<string>,
  currentId: string | null
): string[] {
  const picked = orderedIds.filter((id) => selected.has(id));
  if (picked.length >= 2) return picked.slice(0, mode === 'compare' ? 2 : MAX_PANES);
  if (!currentId) return [];
  const start = orderedIds.indexOf(currentId);
  if (start < 0) return [currentId];
  return orderedIds.slice(start, start + (mode === 'compare' ? 2 : SURVEY_FALLBACK));
}
