import { ORIGINAL_SOURCE, type CompareSource } from '#lib/editor/compare.ts';
import { CompareControl } from '#lib/stores/editor/compareControl.svelte.ts';
import { describe, expect, it, vi } from 'vitest';

type Ctx = {
  geometrySession: object | null;
  snapshots: { items: { id: string; manifest_hash: string }[] };
};

function setup(sideAvailable = true) {
  const ctx: Ctx = {
    geometrySession: null,
    snapshots: {
      items: [
        { id: 'snap-1', manifest_hash: 'h1' },
        { id: 'snap-2', manifest_hash: 'h2' }
      ]
    }
  };
  const side = { available: sideAvailable };
  const changed = vi.fn<(wasOn: boolean) => void>();
  const control = new CompareControl(ctx, changed, () => side.available);
  return { ctx, side, changed, control };
}

const SNAPSHOT: CompareSource = { kind: 'snapshot', id: 'snap-1' };

describe('CompareControl', () => {
  it('cycles split, side by side and off on desktop', () => {
    const { changed, control } = setup();
    const modes = [1, 2, 3].map(() => {
      control.cycle();
      return control.mode;
    });
    expect(modes).toEqual(['split', 'side', 'off']);
    expect(changed.mock.calls).toEqual([[false], [true], [true]]);
  });

  it('cycles only split and off in phone portrait', () => {
    const { control } = setup(false);
    control.cycle();
    expect(control.mode).toBe('split');
    control.cycle();
    expect(control.mode).toBe('off');
  });

  it('shows split while side by side does not fit and returns to it once it does', () => {
    const { side, changed, control } = setup();
    control.open(SNAPSHOT);
    expect(control.mode).toBe('side');
    side.available = false;
    expect(control.mode).toBe('split');
    expect(control.sideBySide).toBe(false);
    side.available = true;
    expect(control.mode).toBe('side');
    expect(changed).toHaveBeenCalledTimes(1);
  });

  it('refuses to start during a geometry session', () => {
    const { ctx, changed, control } = setup();
    ctx.geometrySession = {};
    control.cycle();
    control.open(SNAPSHOT);
    control.setMode('split');
    expect(control.mode).toBe('off');
    expect(changed).not.toHaveBeenCalled();
  });

  it('opens a snapshot side by side, or split where side by side does not fit', () => {
    const desktop = setup();
    desktop.control.open(SNAPSHOT);
    expect([desktop.control.mode, desktop.control.source]).toEqual(['side', SNAPSHOT]);

    const phone = setup(false);
    phone.control.open(SNAPSHOT);
    expect([phone.control.mode, phone.control.source]).toEqual(['split', SNAPSHOT]);
    phone.control.setMode('side');
    expect(phone.control.mode).toBe('split');
  });

  it('switches the left source only while comparing', () => {
    const { changed, control } = setup();
    control.setSource(SNAPSHOT);
    expect(control.source).toBe(ORIGINAL_SOURCE);
    control.cycle();
    control.setSource(SNAPSHOT);
    expect(control.source).toEqual(SNAPSHOT);
    expect(changed.mock.calls).toEqual([[false], [true]]);
    control.setSource({ kind: 'snapshot', id: 'snap-1' });
    expect(changed).toHaveBeenCalledTimes(2);
  });

  it('ends on the original source', () => {
    const { changed, control } = setup();
    control.open(SNAPSHOT);
    control.end();
    expect([control.mode, control.source]).toEqual(['off', ORIGINAL_SOURCE]);
    expect(changed.mock.calls.at(-1)).toEqual([true]);
    control.end();
    expect(changed).toHaveBeenCalledTimes(2);
  });

  it('falls back to the original when the compared snapshot is deleted', () => {
    const { ctx, changed, control } = setup();
    control.open({ kind: 'snapshot', id: 'snap-2' });
    ctx.snapshots.items = [{ id: 'snap-1', manifest_hash: 'h1' }];
    control.snapshotsChanged();
    expect([control.mode, control.source]).toEqual(['side', ORIGINAL_SOURCE]);
    expect(changed.mock.calls.at(-1)).toEqual([true]);
  });

  it('refreshes only when the compared snapshot changes its edits', () => {
    const { ctx, changed, control } = setup();
    control.open(SNAPSHOT);
    changed.mockClear();
    control.snapshotsChanged();
    ctx.snapshots.items = [{ id: 'snap-1', manifest_hash: 'h1' }];
    control.snapshotsChanged();
    expect(changed).not.toHaveBeenCalled();
    ctx.snapshots.items = [{ id: 'snap-1', manifest_hash: 'h1-new' }];
    control.snapshotsChanged();
    expect(changed.mock.calls).toEqual([[true]]);
    expect(control.source).toEqual(SNAPSHOT);
  });

  it('leaves the preview alone for snapshot changes while off', () => {
    const { changed, control } = setup();
    control.snapshotsChanged();
    expect(changed).not.toHaveBeenCalled();
  });

  it('clamps the split position', () => {
    const { control } = setup();
    control.setSplitPos(1.4);
    expect(control.splitPos).toBe(1);
    control.setSplitPos(-0.2);
    expect(control.splitPos).toBe(0);
  });

  it('resets without touching the preview', () => {
    const { changed, control } = setup();
    control.open(SNAPSHOT);
    changed.mockClear();
    control.reset();
    expect([control.mode, control.source]).toEqual(['off', ORIGINAL_SOURCE]);
    expect(changed).not.toHaveBeenCalled();
  });
});
