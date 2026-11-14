import { COPY_TOOLS, toolChanged, type CopyTool } from '#lib/editor/copyTools.ts';
import type { Edits } from '#lib/types/edits.ts';

export interface HistoryChange {
  before: Edits;
  after: Edits;
}

export function lastTouchedTool(change: HistoryChange | null): CopyTool | null {
  if (!change) return null;
  const changed = COPY_TOOLS.filter((tool) => toolChanged(tool, change.before, change.after));
  return changed.length === 1 ? (changed[0] ?? null) : null;
}
