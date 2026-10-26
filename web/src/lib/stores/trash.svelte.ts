import { deleteCopy } from '$lib/api/copies';
import { restoreAssets, trashAssets } from '$lib/api/trash';
import { sourceId } from '$lib/browse/assetKey';
import { countOf } from '$lib/utils/plural';
import {
  chunked,
  planTrash,
  trashBlocked,
  trashedMessage,
  type TrashPlan,
  type TrashTarget
} from '$lib/browse/trash';
import { browseView } from './browseView.svelte';
import { browsing, type Removal } from './browsing.svelte';
import { immichCapabilities } from './immichCapabilities.svelte';
import { selection } from './selection.svelte';
import { toasts } from './toasts.svelte';

const UNDO_TTL_MS = 12000;

function neighbourOf(id: string, gone: ReadonlySet<string>): string | null {
  let next = browsing.nextOf(id);
  while (next && gone.has(next.id)) next = browsing.nextOf(next.id);
  if (next) return next.id;
  let prev = browsing.prevOf(id);
  while (prev && gone.has(prev.id)) prev = browsing.prevOf(prev.id);
  return prev?.id ?? null;
}

function dropFromView(ids: ReadonlySet<string>): Removal {
  const loupe = browseView.loupeId;
  if (loupe && ids.has(loupe)) {
    const next = neighbourOf(loupe, ids);
    if (next) browseView.openLoupe(next);
    else browseView.closeLoupe();
  }
  const active = browseView.activeId;
  if (active && ids.has(active)) browseView.setActive(neighbourOf(active, ids));
  selection.forget(ids);
  return browsing.removeMany(ids);
}

function withVersions(masters: readonly string[]): Set<string> {
  const trashed = new Set(masters);
  for (const asset of browsing.assets) {
    if (asset.copyOf !== undefined && trashed.has(asset.copyOf)) trashed.add(asset.id);
  }
  return trashed;
}

async function inChunks(
  ids: string[],
  send: (chunk: string[]) => Promise<unknown>
): Promise<{ done: string[]; error: unknown }> {
  const done: string[] = [];
  for (const chunk of chunked(ids)) {
    try {
      await send(chunk);
    } catch (error) {
      return { done, error };
    }
    done.push(...chunk);
  }
  return { done, error: null };
}

async function deleteAll(ids: string[]): Promise<string[]> {
  const deleted: string[] = [];
  for (const id of ids) {
    const ok = await deleteCopy(id).then(
      () => true,
      () => false
    );
    if (ok) deleted.push(id);
  }
  return deleted;
}

class TrashStore {
  plan = $state<TrashPlan | null>(null);
  private running = false;
  private settle: ((ok: boolean) => void) | null = null;

  request = async (picked: readonly TrashTarget[]): Promise<boolean> => {
    if (this.plan || this.running) return false;
    const plan = planTrash(picked, browsing.assets);
    if (plan.masters.length > 0 && !immichCapabilities.trash) await immichCapabilities.load();
    const blocked = trashBlocked(plan, immichCapabilities.trash);
    if (blocked) {
      toasts.push('warn', blocked);
      return false;
    }
    this.plan = plan;
    const ok = await new Promise<boolean>((resolve) => {
      this.settle = resolve;
    });
    this.plan = null;
    if (!ok) return false;
    this.running = true;
    try {
      return await this.run(plan);
    } finally {
      this.running = false;
    }
  };

  confirm = (): void => this.finish(true);

  cancel = (): void => this.finish(false);

  removeTrashed = (ids: string[], message: string): void => {
    this.offerUndo(ids, dropFromView(withVersions(ids)), message);
  };

  removeCopy = async (id: string): Promise<void> => {
    try {
      await deleteCopy(id);
    } catch (e) {
      toasts.fail('Delete virtual copy', e);
      return;
    }
    dropFromView(new Set([id]));
    toasts.push('success', 'Virtual copy deleted');
  };

  private finish(ok: boolean): void {
    const settle = this.settle;
    this.settle = null;
    settle?.(ok);
  }

  private async run(plan: TrashPlan): Promise<boolean> {
    const { done: trashed, error } = await inChunks(plan.masters, trashAssets);
    const deleted = await deleteAll(plan.copies);
    const removal = dropFromView(new Set([...withVersions(trashed), ...deleted]));
    if (error) toasts.fail('Move to trash', error);
    const kept = plan.copies.length - deleted.length;
    if (kept > 0) {
      toasts.push(
        'warn',
        `${countOf(kept, 'virtual copy was', 'virtual copies were')} not deleted`
      );
    }
    if (trashed.length === 0 && deleted.length === 0) return false;
    const message = trashedMessage(trashed.length, deleted.length);
    if (trashed.length > 0) this.offerUndo(trashed, removal, message);
    else toasts.push('success', message, UNDO_TTL_MS);
    return true;
  }

  private offerUndo(ids: string[], removal: Removal, message: string): void {
    toasts.push('success', message, UNDO_TTL_MS, {
      label: 'Undo',
      run: () => void this.restore(ids, removal)
    });
  }

  private async restore(ids: string[], removal: Removal): Promise<void> {
    const { done, error } = await inChunks(ids, restoreAssets);
    const restored = new Set(done);
    browsing.reinsert(removal, (asset) => restored.has(sourceId(asset.id)));
    if (error) {
      toasts.fail('Restore from trash', error);
      return;
    }
    toasts.push('success', `Restored ${countOf(done.length, 'photo')} from the Immich trash`);
  }
}

export const trash = new TrashStore();
