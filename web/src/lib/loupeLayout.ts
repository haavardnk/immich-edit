import { neighbourMembers, type MultiMode } from '$lib/compareEntry';

export function paneColumns(count: number): number {
  return count <= 4 ? 2 : 3;
}

export function paneGridStyle(multi: boolean, count: number): string {
  if (!multi || count === 0) return '';
  return `grid-template-columns: repeat(${Math.min(paneColumns(count), count)}, minmax(0, 1fr));`;
}

export function switchMembers(
  mode: MultiMode,
  orderedIds: string[],
  focusedId: string | null,
  members: string[]
): string[] {
  const next =
    mode === 'survey'
      ? neighbourMembers(mode, orderedIds, focusedId)
      : [focusedId, ...members.filter((id) => id !== focusedId)].filter(
          (id): id is string => id !== null
        );
  if (next.length < 2) return [];
  return mode === 'compare' ? next.slice(0, 2) : next;
}
