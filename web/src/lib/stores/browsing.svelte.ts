import type { AssetSummary } from '$lib/types/album';

class BrowsingStore {
  assets = $state<AssetSummary[]>([]);

  set(assets: AssetSummary[]): void {
    this.assets = assets;
  }

  clear(): void {
    this.assets = [];
  }

  patch(id: string, fields: Partial<AssetSummary>): void {
    const idx = this.assets.findIndex((a) => a.id === id);
    if (idx < 0) return;
    this.assets[idx] = { ...this.assets[idx], ...fields };
  }
}

export const browsing = new BrowsingStore();
