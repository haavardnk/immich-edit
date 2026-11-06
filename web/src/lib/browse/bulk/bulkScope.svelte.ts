import { isCopy } from '#lib/browse/assetKey.ts';
import { trashReason } from '#lib/browse/bulk/bulkMarks.ts';
import { scopeLine } from '#lib/browse/bulk/bulkRules.ts';
import { NOT_OWNER, SOME_NOT_OWNER, ownsAsset } from '#lib/browse/ownership.ts';
import { isRejected } from '#lib/browse/reject.ts';
import { labelOf } from '#lib/stores/labels.ts';
import { selection } from '#lib/stores/selection.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import { commonValue, coverage } from '#lib/utils/selectionState.ts';

export type BulkSource = {
  items: () => readonly AssetSummary[];
  selectingAll: () => boolean;
  selectAll: () => Promise<boolean>;
  partial: () => boolean;
  loadingMore: () => boolean;
  total: () => number | undefined;
};

class BulkScope {
  #source = $state.raw<BulkSource | null>(null);

  readonly items = $derived(this.#source?.items() ?? []);
  readonly selectingAll = $derived(this.#source?.selectingAll() ?? false);
  readonly selecting = $derived(
    this.#source !== null && (selection.selecting || this.selectingAll)
  );
  readonly count = $derived(selection.count);
  readonly targetCount = $derived(this.selectingAll ? this.items.length : this.count);
  readonly picked = $derived(this.items.filter((asset) => selection.has(asset.id)));
  readonly single = $derived(
    !this.selectingAll && this.count === 1 ? (this.picked[0] ?? null) : null
  );
  readonly allOwned = $derived(this.picked.every(ownsAsset));
  readonly trashable = $derived(this.picked.some((a) => isCopy(a.id) || ownsAsset(a)));
  readonly trashReason = $derived(trashReason(this.trashable));
  readonly notOwnedReason = $derived(this.count === 1 ? NOT_OWNER : SOME_NOT_OWNER);
  readonly favorite = $derived(coverage(this.picked, this.count, (a) => a.isFavorite));
  readonly rejected = $derived(coverage(this.picked, this.count, (a) => isRejected(a)));
  readonly rating = $derived(commonValue(this.picked, this.count, (a) => a.exifInfo?.rating ?? 0));
  readonly label = $derived(commonValue(this.picked, this.count, (a) => labelOf(a)));
  readonly partial = $derived(this.#source?.partial() ?? false);
  readonly loadingMore = $derived(this.#source?.loadingMore() ?? false);
  readonly canSelectAll = $derived(
    this.partial || this.items.some((asset) => !selection.has(asset.id))
  );
  readonly scope = $derived(scopeLine(this.items.length, this.#source?.total(), this.partial));
  readonly closeLabel = $derived(
    this.count === 0 && !this.selectingAll ? 'Exit selection' : 'Clear selection'
  );

  attach = (source: BulkSource): (() => void) => {
    this.#source = source;
    return () => {
      if (this.#source === source) this.#source = null;
    };
  };

  selectAll = (): Promise<boolean> => this.#source?.selectAll() ?? Promise.resolve(false);
}

export const bulkScope = new BulkScope();
