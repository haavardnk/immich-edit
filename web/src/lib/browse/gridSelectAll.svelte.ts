import { selection } from '#lib/stores/selection.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';

export type SelectAllSource = {
  items: () => readonly AssetSummary[];
  loadingMore: () => boolean;
  loadAll: () => (() => Promise<boolean>) | undefined;
};

export class GridSelectAll {
  busy = $state(false);
  #src: SelectAllSource;

  constructor(src: SelectAllSource) {
    this.#src = src;
  }

  run = async (): Promise<boolean> => {
    if (this.#src.loadingMore() || this.busy) return false;
    this.busy = true;
    try {
      const loadAll = this.#src.loadAll();
      if (loadAll && !(await loadAll())) return false;
      selection.replace(this.#src.items().map((asset) => asset.id));
      return true;
    } finally {
      this.busy = false;
    }
  };
}
