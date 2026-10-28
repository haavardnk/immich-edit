import { listEditedAssets, type EditedAssetEntry } from '$lib/api/edits';

class EditedThumbsStore {
  entries = $state<EditedAssetEntry[]>([]);
  revision = $state('');
  private map = $state(new Map<string, { hash: string; revision?: string }>());
  private hydrated = false;
  private hydrating: Promise<void> | null = null;
  private listenersBound = false;
  private writes = 0;
  private refreshes = 0;

  getHash(id: string): string | undefined {
    return this.map.get(id)?.hash;
  }

  getRevision(id: string): string {
    return this.map.get(id)?.revision || this.revision;
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
    this.refreshes += 1;
    const ticket = this.refreshes;
    const started = this.writes;
    const { render_revision, items } = await listEditedAssets();
    if (ticket !== this.refreshes) return;
    if (started !== this.writes) return this.refresh();
    this.entries = items;
    this.revision = render_revision;
    this.map = new Map(items.map((e) => [e.id, { hash: e.hash, revision: e.revision }]));
  }

  upsert(id: string, hash: string, updatedAt: string): void {
    this.writes += 1;
    const next = new Map(this.map);
    next.set(id, { hash });
    this.map = next;
    const filtered = this.entries.filter((e) => e.id !== id);
    filtered.unshift({ id, hash, updated_at: updatedAt });
    this.entries = filtered;
  }

  remove(id: string): void {
    if (!this.map.has(id)) return;
    this.writes += 1;
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
