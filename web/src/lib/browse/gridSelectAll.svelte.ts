import { selection } from '#lib/stores/selection.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';

export type SelectAllSource = {
  items: () => readonly AssetSummary[];
  loadingMore: () => boolean;
  loadAll: () => ((signal: AbortSignal) => Promise<boolean>) | undefined;
};

export class GridSelectAll {
  busy = $state(false);
  #src: SelectAllSource;
  #abort: AbortController | null = null;

  constructor(src: SelectAllSource) {
    this.#src = src;
  }

  run = async (): Promise<boolean> => {
    if (this.#src.loadingMore() || this.busy) return false;
    const abort = new AbortController();
    this.#abort = abort;
    this.busy = true;
    try {
      const loadAll = this.#src.loadAll();
      if (loadAll && !(await loadAll(abort.signal))) return false;
      if (abort.signal.aborted) return false;
      selection.selectLoaded(this.#src.items().map((asset) => asset.id));
      return true;
    } finally {
      if (this.#abort === abort) {
        this.#abort = null;
        this.busy = false;
      }
    }
  };

  cancel = (): void => {
    this.#abort?.abort();
    this.#abort = null;
    this.busy = false;
  };
}
