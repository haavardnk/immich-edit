import type { EditHistoryEntry } from '#lib/api/edits.ts';
import { neutralEdits } from '#lib/types/edits.ts';
import { describe, expect, it } from 'vitest';
import { currentEntry, entryEdits, stepNumber, stepStates } from './historySteps';

function entry(id: number, undone = false): EditHistoryEntry {
  return {
    id,
    manifest_hash: `h${id}`,
    deleted: false,
    edits: neutralEdits(),
    created_at: '2026-01-01T10:00:00Z',
    action: null,
    undone
  };
}

const ENTRIES = [entry(5, true), entry(4, true), entry(3), entry(2), entry(1)];

describe('history steps', () => {
  it('treats the newest entry that is not undone as current', () => {
    expect(currentEntry(ENTRIES)?.id).toBe(3);
    expect(currentEntry([entry(2), entry(1)])?.id).toBe(2);
    expect(currentEntry([])).toBeNull();
  });

  it('numbers steps from the oldest', () => {
    expect(stepNumber(ENTRIES, 3)).toBe(3);
    expect(stepNumber(ENTRIES, 5)).toBe(5);
    expect(stepNumber(ENTRIES, 1)).toBe(1);
    expect(stepNumber(ENTRIES, 9)).toBe(0);
  });

  it('previews a reset entry as neutral edits and skips entries without edits', () => {
    expect(entryEdits({ ...entry(1), deleted: true, edits: null })).toEqual(neutralEdits());
    expect(entryEdits({ ...entry(1), edits: null })).toBeNull();
  });

  it('supersedes only the steps newer than the hovered one', () => {
    expect(stepStates(ENTRIES, 2).map((state) => state.superseded)).toEqual([
      true,
      true,
      true,
      false,
      false
    ]);
    expect(stepStates(ENTRIES, 2)[3]).toEqual({
      current: false,
      undone: false,
      previewing: true,
      superseded: false
    });
    expect(stepStates(ENTRIES, 5).some((state) => state.superseded)).toBe(false);
    expect(stepStates(ENTRIES, null).map((state) => state.current)).toEqual([
      false,
      false,
      true,
      false,
      false
    ]);
  });
});
