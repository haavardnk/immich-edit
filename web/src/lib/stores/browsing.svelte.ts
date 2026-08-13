import type { AssetSummary } from '$lib/types/album';
import type { SearchQuery } from '$lib/types/search';

class BrowsingStore {
  assets = $state<AssetSummary[]>([]);
  query = $state<SearchQuery | null>(null);
  total = $state<number | undefined>(undefined);

  set(assets: AssetSummary[]): void {
    this.assets = assets;
  }

  setContext(query: SearchQuery | null, total: number | undefined): void {
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

  insertCopy(copyId: string, source: string, label: string | null): void {
    if (this.assets.some((a) => a.id === copyId)) return;
    const start = this.assets.findIndex((a) => a.id === source);
    if (start < 0) return;
    let at = start + 1;
    while (at < this.assets.length && this.assets[at].copyOf === source) at += 1;
    const copy: AssetSummary = {
      ...this.assets[start],
      id: copyId,
      copyOf: source,
      copyLabel: label ?? undefined
    };
    this.assets.splice(at, 0, copy);
  }

  remove(id: string): void {
    const idx = this.assets.findIndex((a) => a.id === id);
    if (idx < 0) return;
    this.assets.splice(idx, 1);
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
