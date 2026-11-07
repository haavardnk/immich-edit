import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { selection } from '#lib/stores/selection.svelte.ts';
import { session } from '#lib/stores/session.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import { bulkScope } from './bulkScope.svelte';
import { bulkActions } from './bulkActions.svelte';

const meta = vi.hoisted(() => ({ setRating: vi.fn(async () => true) }));
const grid = vi.hoisted(() => ({ openMulti: vi.fn() }));
const bin = vi.hoisted(() => ({ request: vi.fn(async () => true) }));

vi.mock('#lib/browse/bulkMeta.ts', () => meta);
vi.mock('#lib/browse/gridActions.ts', () => grid);
vi.mock('#lib/stores/trash.svelte.ts', () => ({ trash: bin }));

function asset(id: string): AssetSummary {
  return {
    id,
    originalFileName: `${id}.jpg`,
    type: 'IMAGE',
    fileCreatedAt: null,
    updatedAt: null,
    checksum: null,
    isFavorite: false,
    exifInfo: null,
    tags: []
  };
}

const ITEMS = [...['a', 'b', 'c', 'd', 'e'].map(asset), { ...asset('f'), ownerId: 'partner' }];
let selectingAll = false;
let detach = (): void => {};
const selectAll = vi.fn(async () => true);
const cancelSelectAll = vi.fn();

beforeEach(() => {
  session.set({ id: 'me', email: 'me@x', name: 'Me', is_admin: false, auth_kind: 'password' });
  selection.clear();
  selectingAll = false;
  detach = bulkScope.attach({
    items: () => ITEMS,
    selectingAll: () => selectingAll,
    selectAll,
    cancelSelectAll,
    partial: () => false,
    loadingMore: () => false,
    total: () => ITEMS.length
  });
});

afterEach(() => {
  detach();
  vi.clearAllMocks();
});

describe('bulkActions', () => {
  it('selects all through the grid so the whole view loads first', async () => {
    await bulkActions.selectAll();
    expect(selectAll).toHaveBeenCalledTimes(1);
  });

  it('does not start select all again while it is running', async () => {
    selectingAll = true;
    await bulkActions.selectAll();
    expect(selectAll).not.toHaveBeenCalled();
  });

  it('writes to the picked photos once at a time', async () => {
    selection.replace(['a', 'b']);
    let release = (): void => {};
    meta.setRating.mockImplementationOnce(
      () => new Promise<boolean>((resolve) => (release = () => resolve(true)))
    );
    const first = bulkActions.setRating(4);
    expect(bulkActions.busy).toBe(true);
    await bulkActions.setRating(2);
    release();
    await first;
    expect(meta.setRating).toHaveBeenCalledTimes(1);
    expect(meta.setRating).toHaveBeenCalledWith(['a', 'b'], 4);
    expect(bulkActions.busy).toBe(false);
  });

  it('opens compare and survey over the listed photos', () => {
    bulkActions.multi('survey');
    expect(grid.openMulti).toHaveBeenCalledWith('survey', ITEMS);
  });

  it('asks to trash only the picked photos', () => {
    selection.replace(['b', 'd']);
    bulkActions.trash();
    expect(bin.request).toHaveBeenCalledWith([ITEMS[1], ITEMS[3]]);
  });

  it('explains instead of acting on photos shared by another user', () => {
    const shared = vi.spyOn(toasts, 'push');
    selection.replace(['f']);
    expect(bulkActions.ownedOrExplain()).toBe(false);
    bulkActions.trash();
    expect(bin.request).not.toHaveBeenCalled();
    expect(shared).toHaveBeenCalledTimes(2);
    selection.replace(['a']);
    expect(bulkActions.ownedOrExplain()).toBe(true);
    shared.mockRestore();
  });

  it('clears the selection and select mode', () => {
    selection.enterMode();
    selection.toggle('a');
    bulkActions.clear();
    expect(selection.selecting).toBe(false);
    expect(cancelSelectAll).not.toHaveBeenCalled();
  });

  it('cancels a running select all and leaves select mode', () => {
    selection.enterMode();
    selection.toggle('a');
    selectingAll = true;
    bulkActions.clear();
    expect(cancelSelectAll).toHaveBeenCalledTimes(1);
    expect([...selection.selected]).toEqual([]);
    expect(selection.mode).toBe(false);
  });
});
