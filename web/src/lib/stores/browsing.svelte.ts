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

  prevOf(id: string): AssetSummary | null {
    const idx = this.assets.findIndex((a) => a.id === id);
    if (idx <= 0) return null;
    return this.assets[idx - 1];
  }

  nextOf(id: string): AssetSummary | null {
    const idx = this.assets.findIndex((a) => a.id === id);
    if (idx < 0 || idx >= this.assets.length - 1) return null;
    return this.assets[idx + 1];
  }
}

export const browsing = new BrowsingStore();
