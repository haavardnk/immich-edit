import type { EditHistoryEntry } from '#lib/api/edits.ts';
import { neutralEdits, type Edits } from '#lib/types/edits.ts';
import { historyLabel } from '#lib/utils/history.ts';

export interface StepState {
  current: boolean;
  undone: boolean;
  previewing: boolean;
  superseded: boolean;
}

export function currentEntry(entries: readonly EditHistoryEntry[]): EditHistoryEntry | null {
  return entries.find((entry) => !entry.undone) ?? null;
}

export function stepNumber(entries: readonly EditHistoryEntry[], entryId: number): number {
  const index = entries.findIndex((entry) => entry.id === entryId);
  return index < 0 ? 0 : entries.length - index;
}

export function stepText(
  entry: EditHistoryEntry,
  previous: EditHistoryEntry | null,
  isRaw: boolean
): string {
  const info = historyLabel(entry, previous, isRaw);
  return info.delta ? `${info.label} ${info.delta}` : info.label;
}

export function entryText(
  entries: readonly EditHistoryEntry[],
  entry: EditHistoryEntry,
  isRaw: boolean
): string {
  return stepText(entry, entries[entries.indexOf(entry) + 1] ?? null, isRaw);
}

export function entryEdits(entry: EditHistoryEntry): Edits | null {
  if (entry.deleted) return neutralEdits();
  return entry.edits;
}

export function stepStates(
  entries: readonly EditHistoryEntry[],
  hoveredId: number | null
): StepState[] {
  const currentId = currentEntry(entries)?.id ?? null;
  const hovered = hoveredId === null ? -1 : entries.findIndex((entry) => entry.id === hoveredId);
  return entries.map((entry, index) => ({
    current: entry.id === currentId,
    undone: entry.undone,
    previewing: index === hovered,
    superseded: hovered > 0 && index < hovered
  }));
}
