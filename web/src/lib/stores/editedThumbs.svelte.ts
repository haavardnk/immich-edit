import { listEditedAssets, type EditedAssetEntry } from '$lib/api/edits';

class EditedThumbsStore {
  entries = $state<EditedAssetEntry[]>([]);
  private map = $state(new Map<string, string>());
  private hydrated = false;
  private hydrating: Promise<void> | null = null;
  private listenersBound = false;

  getHash(id: string): string | undefined {
    return this.map.get(id);
  }

  get count(): number {
    return this.entries.length;
  }

  loadOnce(): Promise<void> {
    if (this.hydrated) return Promise.resolve();
    if (this.hydrating) return this.hydrating;
    this.bindListeners();
    this.hydrating = this.refresh().finally(() => {
      this.hydrated = true;
      this.hydrating = null;
    });
    return this.hydrating;
  }

  async refresh(): Promise<void> {
    const entries = await listEditedAssets();
    this.entries = entries;
    const next = new Map<string, string>();
    for (const e of entries) next.set(e.id, e.hash);
    this.map = next;
  }

  upsert(id: string, hash: string, updatedAt: string): void {
    const next = new Map(this.map);
    next.set(id, hash);
    this.map = next;
    const filtered = this.entries.filter((e) => e.id !== id);
    filtered.unshift({ id, hash, updated_at: updatedAt });
    this.entries = filtered;
  }

  remove(id: string): void {
    if (!this.map.has(id)) return;
    const next = new Map(this.map);
    next.delete(id);
    this.map = next;
    this.entries = this.entries.filter((e) => e.id !== id);
  }

  private bindListeners(): void {
    if (this.listenersBound || typeof window === 'undefined') return;
    this.listenersBound = true;
    window.addEventListener('immich-edit:edits-saved', (ev: Event) => {
      const d = (ev as CustomEvent<{ id: string; hash: string; updated_at: string }>).detail;
      this.upsert(d.id, d.hash, d.updated_at);
    });
    window.addEventListener('immich-edit:edits-deleted', (ev: Event) => {
      const d = (ev as CustomEvent<{ id: string }>).detail;
      this.remove(d.id);
    });
  }
}

export const editedThumbs = new EditedThumbsStore();
