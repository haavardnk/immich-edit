import {
  createSavedView,
  deleteSavedView,
  listSavedViews,
  updateSavedView,
  type SavedView,
  type SavedViewInput
} from '#lib/api/savedViews.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';

const FAIL_TTL_MS = 8000;
const UNDO_TTL_MS = 8000;

function byName(a: SavedView, b: SavedView): number {
  return (
    a.name.localeCompare(b.name, undefined, { sensitivity: 'base' }) ||
    a.created_at.localeCompare(b.created_at)
  );
}

export class SavedViewsStore {
  views = $state<SavedView[]>([]);
  loaded = $state(false);
  #loading: Promise<void> | null = null;

  ensure = (): Promise<void> => {
    this.#loading ??= this.#load();
    return this.#loading;
  };

  #load = async (): Promise<void> => {
    try {
      const views = await listSavedViews();
      this.views = [...views].sort(byName);
      this.loaded = true;
    } catch (e) {
      this.#loading = null;
      toasts.fail('Failed to load saved views', e, FAIL_TTL_MS);
    }
  };

  #put(view: SavedView): void {
    this.views = [...this.views.filter((v) => v.id !== view.id), view].sort(byName);
  }

  create = async (input: SavedViewInput): Promise<SavedView | null> => {
    if (!this.loaded) await this.ensure();
    try {
      const view = await createSavedView(input);
      this.#put(view);
      toasts.push('success', `Saved view “${view.name}”`, { ttlMs: 5000 });
      return view;
    } catch (e) {
      toasts.fail('Failed to save view', e, FAIL_TTL_MS);
      return null;
    }
  };

  rename = async (view: SavedView, name: string): Promise<boolean> => {
    if (!this.loaded) await this.ensure();
    try {
      this.#put(await updateSavedView(view.id, { name, url: view.url }));
      return true;
    } catch (e) {
      toasts.fail('Failed to rename view', e, FAIL_TTL_MS);
      return false;
    }
  };

  remove = async (view: SavedView): Promise<void> => {
    if (!this.loaded) await this.ensure();
    this.views = this.views.filter((v) => v.id !== view.id);
    try {
      await deleteSavedView(view.id);
      toasts.push('success', `Deleted “${view.name}”`, {
        ttlMs: UNDO_TTL_MS,
        action: { label: 'Undo', run: () => void this.restore(view) }
      });
    } catch (e) {
      this.#put(view);
      toasts.fail('Failed to delete view', e, FAIL_TTL_MS);
    }
  };

  restore = async (view: SavedView): Promise<void> => {
    if (!this.loaded) await this.ensure();
    try {
      this.#put(await createSavedView({ name: view.name, url: view.url }));
    } catch (e) {
      toasts.fail('Failed to restore view', e, FAIL_TTL_MS);
    }
  };
}

export const savedViews = new SavedViewsStore();
