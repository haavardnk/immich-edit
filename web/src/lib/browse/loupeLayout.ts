import { neighbourMembers, type MultiMode } from './compareEntry';

export function paneColumns(count: number, stacked = false): number {
  if (stacked && count <= 2) return 1;
  return count <= 4 ? 2 : 3;
}

export function paneWindow(
  count: number,
  focus: number,
  limit: number
): { start: number; end: number } {
  const size = Math.min(count, Math.max(1, limit));
  const start = Math.min(Math.max(0, focus - size + 1), count - size);
  return { start, end: start + size };
}

export const INFO_SHEET_SHARE = 0.55;

export const INFO_SHEET_INSET = `padding-bottom: calc(${INFO_SHEET_SHARE} * (var(--app-height) - var(--safe-top) - var(--sheet-gap)));`;

export function paneGridStyle(multi: boolean, count: number, stacked: boolean): string {
  if (!multi || count === 0) return '';
  const columns = Math.min(paneColumns(count, stacked), count);
  const rows = Math.ceil(count / columns);
  return `grid-template-columns: repeat(${columns}, minmax(0, 1fr)); grid-template-rows: repeat(${rows}, minmax(0, 1fr));`;
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
