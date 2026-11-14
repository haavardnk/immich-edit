import { compare, COMPARE_PANES, MAX_PANES, type CompareMode } from '#lib/stores/compare.svelte.ts';

export type MultiMode = Exclude<CompareMode, 'single'>;

const SURVEY_FALLBACK = 6;

export function surveyUnavailable(): string | null {
  return compare.surveyAvailable ? null : 'Survey needs a larger screen';
}

export function multiMembers(
  mode: MultiMode,
  orderedIds: string[],
  selected: ReadonlySet<string>,
  currentId: string | null
): string[] {
  const picked = orderedIds.filter((id) => selected.has(id));
  if (picked.length >= 2) return picked.slice(0, mode === 'compare' ? COMPARE_PANES : MAX_PANES);
  return neighbourMembers(mode, orderedIds, currentId);
}

export function neighbourMembers(
  mode: MultiMode,
  orderedIds: string[],
  currentId: string | null
): string[] {
  if (!currentId) return [];
  const start = orderedIds.indexOf(currentId);
  if (start < 0) return [currentId];
  return orderedIds.slice(start, start + (mode === 'compare' ? COMPARE_PANES : SURVEY_FALLBACK));
}
