import type { EditSnapshot } from '#lib/api/snapshots.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import { layout, layoutControls, renderLayout, settle } from '#lib/testing/layoutParity.ts';
import { neutralEdits, type Edits } from '#lib/types/edits.ts';
import { flushSync } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import HistoryPopover from '../history/HistoryPopover.svelte';
import SnapshotList from './SnapshotList.svelte';
import SnapshotsSheet from './SnapshotsSheet.svelte';

const api = vi.hoisted(() => ({
  listSnapshots: vi.fn(),
  createSnapshot: vi.fn(),
  updateSnapshot: vi.fn(),
  deleteSnapshot: vi.fn(),
  restoreSnapshot: vi.fn(),
  listEditHistory: vi.fn()
}));

vi.mock('#lib/api/snapshots.ts', () => ({
  listSnapshots: api.listSnapshots,
  createSnapshot: api.createSnapshot,
  updateSnapshot: api.updateSnapshot,
  deleteSnapshot: api.deleteSnapshot,
  restoreSnapshot: api.restoreSnapshot
}));

vi.mock('#lib/api/edits.ts', async (importOriginal) => ({
  ...(await importOriginal<typeof import('#lib/api/edits.ts')>()),
  listEditHistory: api.listEditHistory
}));

function snapshot(id: string, name: string): EditSnapshot {
  return {
    id,
    name,
    manifest_hash: `hash-${id}`,
    edits: neutralEdits(),
    created_at: '2026-11-10T10:00:00Z',
    updated_at: '2026-11-10T10:00:00Z'
  };
}

function find(selector: string): HTMLElement {
  const target = document.body.querySelector<HTMLElement>(selector);
  if (!target) throw new Error(`Missing ${selector}`);
  return target;
}

function click(selector: string): void {
  find(selector).click();
  flushSync();
}

beforeEach(async () => {
  api.listSnapshots.mockResolvedValue([snapshot('s1', 'Warm v1'), snapshot('s2', 'Snapshot 1')]);
  api.listEditHistory.mockResolvedValue([]);
  editor.assetId = 'a1';
  editor.initialised = true;
  await editor.snapshots.load('a1');
});

afterEach(() => {
  editor.snapshots.reset();
  editor.assetId = null;
  editor.initialised = false;
  for (const mock of Object.values(api)) mock.mockReset();
});

describe('SnapshotList', () => {
  it('exposes the same controls standalone, in the history popover and in the sheet', async () => {
    const found = await layoutControls([
      layout('body', SnapshotList, {}),
      layout(
        'popover',
        HistoryPopover,
        {},
        {
          open: () => click('button[aria-label="Edit history"]'),
          exclude: [
            { role: 'button', name: 'Edit history' },
            { role: 'button', name: 'Close' }
          ]
        }
      ),
      layout(
        'sheet',
        SnapshotsSheet,
        { open: true, onOpenChange: () => undefined },
        {
          exclude: [
            { role: 'button', name: 'Close' },
            { role: 'button', name: 'Resize sheet' }
          ]
        }
      )
    ]);
    expect(found.body.map((control) => control.name)).toEqual(
      expect.arrayContaining(['New snapshot', 'Restore Warm v1', 'Rename Warm v1'])
    );
    expect(found.popover).toEqual(found.body);
    expect(found.sheet).toEqual(found.body);
  });

  it('names a new snapshot inline right after creating it', async () => {
    api.createSnapshot.mockImplementation((_asset: string, name: string, edits: Edits) =>
      Promise.resolve({ ...snapshot('s3', name), edits })
    );
    const destroy = renderLayout(layout('body', SnapshotList, {}));
    await settle();

    click('button[aria-label="New snapshot"]');
    await settle();

    expect(api.createSnapshot).toHaveBeenCalledWith('a1', 'Snapshot 2', expect.any(Object));
    const field = find('input[aria-label="Snapshot name"]') as HTMLInputElement;
    expect(field.value).toBe('Snapshot 2');
    expect(document.activeElement).toBe(field);
    await destroy();
  });
});
