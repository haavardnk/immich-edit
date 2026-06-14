import { describe, it, expect } from 'vitest';
import { historyLabel } from './history';
import { neutralEdits } from '$lib/types/edits';
import type { EditHistoryEntry } from '$lib/api/edits';

function entry(over: Partial<EditHistoryEntry>): EditHistoryEntry {
  return {
    id: 1,
    manifest_hash: 'abcdef1234567890',
    deleted: false,
    edits: neutralEdits(),
    created_at: '2024-01-01T00:00:00Z',
    action: null,
    ...over
  };
}

describe('historyLabel', () => {
  it('labels a deletion with its action or a reset fallback', () => {
    expect(historyLabel(entry({ deleted: true, action: 'Reset' }), null).label).toBe('Reset');
    expect(historyLabel(entry({ deleted: true, action: null }), null).label).toBe(
      'Reset to original'
    );
  });

  it('names a single changed field with its delta', () => {
    const curr = neutralEdits();
    curr.basic.exposure_ev = 1.5;
    const out = historyLabel(entry({ edits: curr }), null);
    expect(out.label).toBe('Exposure');
    expect(out.delta).toBe('+1.50');
  });

  it('formats a negative single-field delta', () => {
    const curr = neutralEdits();
    curr.tone.shadows = -20;
    const out = historyLabel(entry({ edits: curr }), null);
    expect(out.label).toBe('Shadows');
    expect(out.delta).toBe('-20');
  });

  it('reports multiple changes when more than one field differs', () => {
    const curr = neutralEdits();
    curr.basic.exposure_ev = 1;
    curr.basic.contrast = 10;
    const out = historyLabel(entry({ edits: curr, action: null }), null);
    expect(out.label).toBe('Multiple changes');
  });

  it('prefers an explicit action over the multi-change label', () => {
    const curr = neutralEdits();
    curr.basic.exposure_ev = 1;
    curr.basic.contrast = 10;
    const out = historyLabel(entry({ edits: curr, action: 'Pasted edits' }), null);
    expect(out.label).toBe('Pasted edits');
  });

  it('diffs against the previous entry, not neutral', () => {
    const prev = neutralEdits();
    prev.basic.exposure_ev = 1;
    const curr = neutralEdits();
    curr.basic.exposure_ev = 1.5;
    const out = historyLabel(
      entry({ id: 2, edits: curr }),
      entry({ id: 1, edits: prev })
    );
    expect(out.label).toBe('Exposure');
    expect(out.delta).toBe('+0.50');
  });

  it('falls back to the hash when nothing changed', () => {
    const out = historyLabel(entry({ edits: neutralEdits(), action: null }), null);
    expect(out.label).toBe('abcdef12');
  });
});
