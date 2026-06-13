import type { AssetSummary } from '$lib/types/album';

class BrowsingStore {
  assets = $state<AssetSummary[]>([]);
  query = $state<Record<string, unknown> | null>(null);
  total = $state<number | undefined>(undefined);

  set(assets: AssetSummary[]): void {
    this.assets = assets;
  }

  setContext(query: Record<string, unknown> | null, total: number | undefined): void {
    this.query = query;
    this.total = total;
  }

  clear(): void {
    this.assets = [];
    this.query = null;
    this.total = undefined;
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
