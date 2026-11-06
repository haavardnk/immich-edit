import { beforeEach, describe, expect, it, vi } from 'vitest';
import {
  createSavedView,
  deleteSavedView,
  listSavedViews,
  updateSavedView,
  type SavedView
} from '#lib/api/savedViews.ts';
import { SavedViewsStore } from '#lib/stores/savedViews.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';

vi.mock('#lib/api/savedViews.ts', () => ({
  listSavedViews: vi.fn(),
  createSavedView: vi.fn(),
  updateSavedView: vi.fn(),
  deleteSavedView: vi.fn()
}));

function view(id: string, name: string, created = '2026-01-01T00:00:00Z'): SavedView {
  return { id, name, url: `/photos?tag=${id}`, created_at: created, updated_at: created };
}

const names = (store: SavedViewsStore) => store.views.map((v) => v.name);

beforeEach(() => {
  vi.mocked(listSavedViews).mockReset();
  vi.mocked(createSavedView).mockReset();
  vi.mocked(updateSavedView).mockReset();
  vi.mocked(deleteSavedView).mockReset();
  toasts.items = [];
});

describe('saved views store', () => {
  it('loads once and sorts by name, then age', async () => {
    vi.mocked(listSavedViews).mockResolvedValue([
      view('b', 'beta'),
      view('a2', 'Alpha', '2026-02-01T00:00:00Z'),
      view('a1', 'alpha', '2026-01-01T00:00:00Z')
    ]);
    const store = new SavedViewsStore();
    await Promise.all([store.ensure(), store.ensure()]);
    expect(listSavedViews).toHaveBeenCalledTimes(1);
    expect(store.views.map((v) => v.id)).toEqual(['a1', 'a2', 'b']);
    expect(store.loaded).toBe(true);
  });

  it('retries a failed load', async () => {
    vi.mocked(listSavedViews).mockRejectedValueOnce(new Error('down')).mockResolvedValue([]);
    const store = new SavedViewsStore();
    await store.ensure();
    expect(store.loaded).toBe(false);
    expect(toasts.items.at(-1)?.message).toBe('Failed to load saved views: down');
    await store.ensure();
    expect(store.loaded).toBe(true);
  });

  it('adds a created view in order and confirms it', async () => {
    vi.mocked(listSavedViews).mockResolvedValue([view('a', 'Alpha'), view('c', 'Gamma')]);
    vi.mocked(createSavedView).mockResolvedValue(view('b', 'Beta'));
    const store = new SavedViewsStore();
    await store.ensure();
    const created = await store.create({ name: 'Beta', url: '/photos' });
    expect(created?.id).toBe('b');
    expect(names(store)).toEqual(['Alpha', 'Beta', 'Gamma']);
    expect(toasts.items.at(-1)).toMatchObject({ kind: 'success', message: 'Saved view “Beta”' });
  });

  it('reports a failed create and returns nothing', async () => {
    vi.mocked(listSavedViews).mockResolvedValue([]);
    vi.mocked(createSavedView).mockRejectedValue(new Error('too long'));
    const store = new SavedViewsStore();
    expect(await store.create({ name: 'x', url: '/photos' })).toBeNull();
    expect(store.views).toEqual([]);
    expect(toasts.items.at(-1)?.message).toBe('Failed to save view: too long');
  });

  it('renames a view keeping its url', async () => {
    vi.mocked(listSavedViews).mockResolvedValue([view('a', 'Alpha'), view('b', 'Beta')]);
    vi.mocked(updateSavedView).mockResolvedValue({ ...view('a', 'Zulu') });
    const store = new SavedViewsStore();
    await store.ensure();
    const first = store.views[0];
    if (!first) throw new Error('no view');
    expect(await store.rename(first, 'Zulu')).toBe(true);
    expect(updateSavedView).toHaveBeenCalledWith('a', { name: 'Zulu', url: '/photos?tag=a' });
    expect(names(store)).toEqual(['Beta', 'Zulu']);
  });

  it('deletes at once and restores through the undo action', async () => {
    const alpha = view('a', 'Alpha');
    vi.mocked(listSavedViews).mockResolvedValue([alpha, view('b', 'Beta')]);
    vi.mocked(deleteSavedView).mockResolvedValue(undefined);
    vi.mocked(createSavedView).mockResolvedValue(view('a-new', 'Alpha'));
    const store = new SavedViewsStore();
    await store.ensure();
    const removal = store.remove(alpha);
    expect(names(store)).toEqual(['Beta']);
    await removal;
    const toast = toasts.items.at(-1);
    expect(toast?.message).toBe('Deleted “Alpha”');
    toast?.action?.run();
    await vi.waitFor(() => expect(names(store)).toEqual(['Alpha', 'Beta']));
    expect(createSavedView).toHaveBeenCalledWith({ name: 'Alpha', url: '/photos?tag=a' });
  });

  it('puts a view back when deleting fails', async () => {
    const alpha = view('a', 'Alpha');
    vi.mocked(listSavedViews).mockResolvedValue([alpha]);
    vi.mocked(deleteSavedView).mockRejectedValue(new Error('gone'));
    const store = new SavedViewsStore();
    await store.ensure();
    await store.remove(alpha);
    expect(names(store)).toEqual(['Alpha']);
    expect(toasts.items.at(-1)?.message).toBe('Failed to delete view: gone');
  });

  it('keeps a view created while the first load is still running', async () => {
    let finish: (views: SavedView[]) => void = () => undefined;
    vi.mocked(listSavedViews).mockReturnValue(new Promise((resolve) => (finish = resolve)));
    vi.mocked(createSavedView).mockResolvedValue(view('b', 'Beta'));
    const store = new SavedViewsStore();
    const loading = store.ensure();
    const creating = store.create({ name: 'Beta', url: '/photos' });
    await Promise.resolve();
    expect(createSavedView).not.toHaveBeenCalled();
    finish([view('a', 'Alpha')]);
    await Promise.all([loading, creating]);
    expect(names(store)).toEqual(['Alpha', 'Beta']);
  });

  it('does not bring back a view deleted while the first load is still running', async () => {
    const alpha = view('a', 'Alpha');
    let finish: (views: SavedView[]) => void = () => undefined;
    vi.mocked(listSavedViews).mockReturnValue(new Promise((resolve) => (finish = resolve)));
    vi.mocked(deleteSavedView).mockResolvedValue(undefined);
    const store = new SavedViewsStore();
    void store.ensure();
    const removal = store.remove(alpha);
    finish([alpha, view('b', 'Beta')]);
    await removal;
    expect(deleteSavedView).toHaveBeenCalledWith('a');
    expect(names(store)).toEqual(['Beta']);
  });
});
