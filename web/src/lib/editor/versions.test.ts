import type { EditHistoryEntry } from '#lib/api/edits.ts';
import { describe, expect, it } from 'vitest';
import { copyVersion, versionsOf, versionSummary, type Version } from './versions';

const MASTER = '00000000-0000-0000-0000-000000000001';

function entry(id: number, action: string, undone = false): EditHistoryEntry {
  return {
    id,
    manifest_hash: `hash-${id}`,
    deleted: false,
    edits: null,
    created_at: '2026-01-01T10:00:00Z',
    action,
    undone
  };
}

describe('versionsOf', () => {
  it('lists the original first and names unnamed copies by index', () => {
    const versions = versionsOf(MASTER, [
      { id: `${MASTER}_2`, source_asset_id: MASTER, name: 'B&W', created_at: '' },
      { id: `${MASTER}_3`, source_asset_id: MASTER, name: null, created_at: '' }
    ]);
    expect(versions).toEqual([
      { id: MASTER, name: 'Original', copyIndex: null, named: false },
      { id: `${MASTER}_2`, name: 'B&W', copyIndex: 2, named: true },
      { id: `${MASTER}_3`, name: 'Copy 3', copyIndex: 3, named: false }
    ]);
  });
});

describe('versionSummary', () => {
  const original: Version = { id: MASTER, name: 'Original', copyIndex: null, named: false };
  const named = copyVersion({
    id: `${MASTER}_2`,
    source_asset_id: MASTER,
    name: 'B&W',
    created_at: ''
  });
  const unnamed = copyVersion({
    id: `${MASTER}_3`,
    source_asset_id: MASTER,
    name: null,
    created_at: ''
  });
  const entries = [entry(3, 'Redone', true), entry(2, 'Portra soft'), entry(1, 'Auto')];

  it('shows the last applied action and the step count', () => {
    expect(versionSummary(original, entries, true)).toBe('Portra soft · 3 steps');
  });

  it('prefixes named copies with their copy number', () => {
    expect(versionSummary(named, entries, true)).toBe('Copy 2 · Portra soft · 3 steps');
  });

  it('skips the copy number when the name already carries it', () => {
    expect(versionSummary(unnamed, [entry(1, 'Crop 1:1')], true)).toBe('Crop 1:1 · 1 step');
  });

  it('says when a version has no edits', () => {
    expect(versionSummary(original, [], true)).toBe('No edits');
  });

  it('counts steps when every step is undone', () => {
    expect(versionSummary(original, [entry(1, 'Auto', true)], true)).toBe('1 step');
  });

  it('shows only the copy number while the history is loading', () => {
    expect(versionSummary(named, undefined, true)).toBe('Copy 2');
    expect(versionSummary(original, undefined, true)).toBe('');
  });
});
