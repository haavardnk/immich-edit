import type { EditHistoryEntry } from '#lib/api/edits.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { neutralEdits } from '#lib/types/edits.ts';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const api = vi.hoisted(() => ({ listEditHistory: vi.fn() }));

vi.mock('#lib/api/edits.ts', () => ({ listEditHistory: api.listEditHistory }));

import { HistorySteps, type HistoryStepsCtx } from '#lib/stores/editor/historySteps.svelte.ts';

type Ctx = { -readonly [K in keyof HistoryStepsCtx]: HistoryStepsCtx[K] };

function entry(id: number, undone = false): EditHistoryEntry {
  return {
    id,
    manifest_hash: `h${id}`,
    deleted: false,
    edits: neutralEdits(),
    created_at: '2026-01-01T10:00:00Z',
    action: `Step ${id}`,
    undone
  };
}

function context(): Ctx {
  const ctx: Ctx = {
    assetId: 'asset-1',
    isRaw: false,
    savedHash: 'h3',
    restoreCount: 0,
    settled: vi.fn(() => Promise.resolve()),
    restoreHistoryEntry: vi.fn(() => {
      ctx.restoreCount++;
      return Promise.resolve(true);
    }),
    previewEdits: vi.fn(),
    endEditsPreview: vi.fn()
  };
  return ctx;
}

describe('HistorySteps', () => {
  let ctx: Ctx;
  let steps: HistorySteps;

  beforeEach(async () => {
    toasts.items = [];
    api.listEditHistory.mockReset();
    api.listEditHistory.mockResolvedValue([entry(4, true), entry(3), entry(2), entry(1)]);
    ctx = context();
    steps = new HistorySteps(ctx);
    await steps.refresh();
  });

  it('shows the current step position while the sheet is open', () => {
    expect(steps.pill).toBeNull();
    steps.sheetOpen = true;
    expect(steps.pill).toEqual({
      lead: 'Step 3 of 4 ·',
      label: 'Step 3',
      tail: '',
      mono: false
    });
  });

  it('previews a hovered step through the history owner', () => {
    const older = steps.entries[2];
    if (!older) throw new Error('missing entry');
    steps.hover(older);
    expect(ctx.previewEdits).toHaveBeenCalledWith('history', older.edits);
    expect(steps.pill?.lead).toBe('Previewing');

    steps.leave();
    expect(ctx.endEditsPreview).toHaveBeenCalledWith('history');
    expect(steps.hoveredId).toBeNull();
  });

  it('does not preview the current step', () => {
    const current = steps.current;
    if (!current) throw new Error('missing current');
    steps.hover(current);
    expect(ctx.previewEdits).not.toHaveBeenCalled();
  });

  it('jumps from the current entry with its edits and ends the preview', async () => {
    const older = steps.entries[3];
    if (!older) throw new Error('missing entry');
    steps.hover(older);
    await steps.restore(older);

    expect(ctx.endEditsPreview).toHaveBeenCalledWith('history');
    expect(ctx.settled).toHaveBeenCalledTimes(1);
    expect(ctx.restoreHistoryEntry).toHaveBeenCalledWith({
      id: 1,
      edits: older.edits,
      from: 3,
      label: 'Step 1'
    });
    expect(api.listEditHistory).toHaveBeenCalledTimes(2);
  });

  it('jumps from the step that is current once pending saves land', async () => {
    vi.mocked(ctx.settled).mockImplementationOnce(() => {
      ctx.savedHash = 'h5';
      return Promise.resolve();
    });
    api.listEditHistory.mockResolvedValueOnce([entry(5), entry(3), entry(2), entry(1)]);
    const older = steps.entries[3];
    if (!older) throw new Error('missing entry');
    await steps.restore(older);
    expect(ctx.restoreHistoryEntry).toHaveBeenCalledWith(
      expect.objectContaining({ id: 1, from: 5 })
    );
  });

  it('skips a jump to the step that became current while saves landed', async () => {
    vi.mocked(ctx.settled).mockImplementationOnce(() => {
      ctx.savedHash = 'h1';
      return Promise.resolve();
    });
    api.listEditHistory.mockResolvedValueOnce([entry(3, true), entry(2, true), entry(1)]);
    const older = steps.entries[3];
    if (!older) throw new Error('missing entry');
    await steps.restore(older);
    expect(ctx.restoreHistoryEntry).not.toHaveBeenCalled();
    expect(steps.busyId).toBeNull();
  });

  it('fetches once per saved state and restore and shares a fetch in flight', async () => {
    await steps.refresh();
    expect(api.listEditHistory).toHaveBeenCalledTimes(1);

    ctx.savedHash = 'h4';
    await Promise.all([steps.refresh(), steps.refresh()]);
    expect(api.listEditHistory).toHaveBeenCalledTimes(2);

    ctx.restoreCount++;
    await Promise.all([steps.refresh(), steps.refresh()]);
    expect(api.listEditHistory).toHaveBeenCalledTimes(3);
    await steps.refresh();
    expect(api.listEditHistory).toHaveBeenCalledTimes(3);
  });

  it('reports a failed jump', async () => {
    vi.mocked(ctx.restoreHistoryEntry).mockRejectedValueOnce(new Error('gone'));
    const older = steps.entries[3];
    if (!older) throw new Error('missing entry');
    await steps.restore(older);
    expect(toasts.items.at(-1)?.message).toBe('Could not restore the history step: gone');
    expect(steps.busyId).toBeNull();
  });

  it('ignores a stale refresh after a reset', async () => {
    let resolve: (entries: EditHistoryEntry[]) => void = () => undefined;
    api.listEditHistory.mockReturnValueOnce(new Promise((done) => (resolve = done)));
    const pending = steps.refresh();
    steps.reset();
    resolve([entry(9)]);
    await pending;
    expect(steps.entries).toEqual([]);
  });
});
