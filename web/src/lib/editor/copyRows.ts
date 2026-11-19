import type { CopyTool, CopyToolRow } from '#lib/editor/copyTools.ts';
import { PHOTO_TOOLS } from '#lib/editor/copyTools.ts';
import { modifiedSections, modifiedTabCount } from '#lib/editor/modified.ts';
import type { Edits } from '#lib/types/edits.ts';
import { countOf } from '#lib/utils/plural.ts';

export interface CopyHints {
  modified: ReadonlySet<CopyTool>;
  touched: CopyTool | null;
  masks: number;
  retouch: number;
}

export function modifiedTools(edits: Edits): Set<CopyTool> {
  const tools = new Set<CopyTool>(modifiedSections(edits));
  for (const tool of PHOTO_TOOLS) if (modifiedTabCount(edits, tool) > 0) tools.add(tool);
  return tools;
}

export function copyHints(edits: Edits | null, touched: CopyTool | null): CopyHints {
  return {
    modified: edits ? modifiedTools(edits) : new Set(),
    touched,
    masks: edits?.masks.length ?? 0,
    retouch: edits?.retouch.length ?? 0
  };
}

export function rowModified(row: CopyToolRow, hints: CopyHints): boolean {
  return row.tools.some((tool) => hints.modified.has(tool));
}

export function rowDot(row: CopyToolRow, hints: CopyHints, nested: boolean): boolean {
  return !nested && row.key !== 'masks' && row.key !== 'retouch' && rowModified(row, hints);
}

export function rowNote(row: CopyToolRow, hints: CopyHints, nested: boolean): string | null {
  if (row.children.length === 0 && hints.touched && row.tools.includes(hints.touched)) {
    return 'last touched';
  }
  if (row.key === 'masks') return hints.masks > 0 ? countOf(hints.masks, 'layer') : null;
  if (row.key === 'retouch') return hints.retouch > 0 ? countOf(hints.retouch, 'stroke') : null;
  if (nested || row.key === 'geometry' || rowModified(row, hints)) return null;
  return 'not modified';
}

export function visibleRows(
  rows: readonly CopyToolRow[],
  available: readonly CopyTool[]
): CopyToolRow[] {
  return rows.flatMap((row) => {
    const tools = row.tools.filter((tool) => available.includes(tool));
    if (tools.length === 0) return [];
    const children = row.children.filter((child) => child.tools.some((t) => tools.includes(t)));
    return [{ ...row, tools, children }];
  });
}

function lines(row: CopyToolRow): number {
  return 1 + row.children.length;
}

export function splitColumns(
  develop: readonly CopyToolRow[],
  photo: readonly CopyToolRow[]
): [CopyToolRow[], CopyToolRow[]] {
  const developLines = develop.reduce((sum, row) => sum + lines(row), 0);
  const total = 1 + developLines + (photo.length > 0 ? 1 + photo.length : 0);
  let left = 1;
  let split = 0;
  let best = Math.max(left, total - left);
  develop.forEach((row, index) => {
    left += lines(row);
    const tallest = Math.max(left, total - left);
    if (tallest < best) {
      best = tallest;
      split = index + 1;
    }
  });
  return [develop.slice(0, split), develop.slice(split)];
}
