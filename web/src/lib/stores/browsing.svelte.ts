import type { AssetSummary } from '$lib/types/album';

class BrowsingStore {
  assets = $state<AssetSummary[]>([]);

  set(assets: AssetSummary[]): void {
    this.assets = assets;
  }

  clear(): void {
    this.assets = [];
  }
}

export const browsing = new BrowsingStore();
