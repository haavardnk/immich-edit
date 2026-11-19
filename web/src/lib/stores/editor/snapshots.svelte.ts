import {
  createSnapshot,
  deleteSnapshot,
  listSnapshots,
  updateSnapshot,
  type EditSnapshot
} from '#lib/api/snapshots.ts';
import { nextSnapshotName } from '#lib/editor/snapshotName.ts';
import { SnapshotThumbs } from '#lib/stores/editor/snapshotThumbs.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import type { Edits } from '#lib/types/edits.ts';

export interface SnapshotCtx {
  assetId: string | null;
  edits: Edits;
  savedHash: string;
  restoreSnapshot(id: string, name: string): Promise<boolean>;
  readonly compare: { snapshotsChanged(): void };
}

export class SnapshotStore {
  items = $state<EditSnapshot[]>([]);
  loading = $state(false);
  readonly thumbs = new SnapshotThumbs();
  private restored = $state<string | null>(null);
  private assetId: string | null = null;
  private ctx: SnapshotCtx;

  constructor(ctx: SnapshotCtx) {
    this.ctx = ctx;
  }

  get restoredId(): string | null {
    const snapshot = this.items.find((item) => item.id === this.restored);
    return snapshot && snapshot.manifest_hash === this.ctx.savedHash ? snapshot.id : null;
  }

  get nextName(): string {
    return nextSnapshotName(this.items.map((item) => item.name));
  }

  async load(assetId: string): Promise<void> {
    if (this.assetId !== assetId) this.thumbs.reset();
    this.assetId = assetId;
    this.items = [];
    this.restored = null;
    this.loading = true;
    try {
      const items = await listSnapshots(assetId);
      if (this.assetId === assetId) this.items = items;
    } catch {
      if (this.assetId === assetId) this.items = [];
    } finally {
      if (this.assetId === assetId) this.loading = false;
    }
  }

  reset(): void {
    this.assetId = null;
    this.items = [];
    this.restored = null;
    this.loading = false;
    this.thumbs.reset();
  }

  async create(name?: string): Promise<EditSnapshot | null> {
    const assetId = this.assetId;
    if (!assetId) return null;
    const edits = $state.snapshot(this.ctx.edits) as Edits;
    try {
      const created = await createSnapshot(assetId, name?.trim() || this.nextName, edits);
      if (this.assetId === assetId) this.items = [created, ...this.items];
      return created;
    } catch (e) {
      toasts.fail('Could not save the snapshot', e);
      return null;
    }
  }

  rename(id: string, name: string): Promise<boolean> {
    const trimmed = name.trim();
    if (!trimmed) return Promise.resolve(false);
    if (this.items.find((item) => item.id === id)?.name === trimmed) return Promise.resolve(true);
    return this.update(id, { name: trimmed }, 'Could not rename the snapshot');
  }

  async updateWithCurrent(id: string): Promise<boolean> {
    const edits = $state.snapshot(this.ctx.edits) as Edits;
    if (!(await this.update(id, { edits }, 'Could not update the snapshot'))) return false;
    const name = this.items.find((item) => item.id === id)?.name;
    if (name) toasts.push('success', `Updated ${name}`, { ttlMs: 3000 });
    return true;
  }

  async remove(id: string): Promise<boolean> {
    const assetId = this.assetId;
    if (!assetId) return false;
    try {
      await deleteSnapshot(assetId, id);
      if (this.assetId === assetId) {
        this.items = this.items.filter((item) => item.id !== id);
        this.ctx.compare.snapshotsChanged();
      }
      if (this.restored === id) this.restored = null;
      return true;
    } catch (e) {
      toasts.fail('Could not delete the snapshot', e);
      return false;
    }
  }

  async restore(id: string): Promise<boolean> {
    const snapshot = this.items.find((item) => item.id === id);
    if (!snapshot) return false;
    try {
      if (!(await this.ctx.restoreSnapshot(id, snapshot.name))) return false;
    } catch (e) {
      toasts.fail('Could not restore the snapshot', e);
      return false;
    }
    this.restored = id;
    return true;
  }

  private async update(
    id: string,
    change: { name?: string; edits?: Edits },
    failure: string
  ): Promise<boolean> {
    const assetId = this.assetId;
    if (!assetId) return false;
    try {
      const updated = await updateSnapshot(assetId, id, change);
      if (this.assetId === assetId) {
        this.items = this.items.map((item) => (item.id === id ? updated : item));
        this.ctx.compare.snapshotsChanged();
      }
      return true;
    } catch (e) {
      toasts.fail(failure, e);
      return false;
    }
  }
}
