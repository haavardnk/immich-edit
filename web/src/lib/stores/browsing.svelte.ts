import { browseControls } from '$lib/stores/browseControls.svelte';
import type { AssetSummary } from '$lib/types/album';

const PREFETCH_WINDOW = 10;

export interface BrowsePager {
  readonly hasMore: boolean;
  readonly loadingMore: boolean;
  loadMore(): Promise<boolean>;
}

class BrowsingStore {
  assets = $state<AssetSummary[]>([]);
  total = $state<number | undefined>(undefined);
  readonly visible = $derived(this.assets.filter((asset) => this.shows(asset)));
  private pager = $state.raw<BrowsePager | null>(null);

  get count(): number {
    return Math.max(
      (this.total ?? 0) - (this.assets.length - this.visible.length),
      this.visible.length
    );
  }

  get hasMore(): boolean {
    return this.pager?.hasMore ?? false;
  }

  get loadingMore(): boolean {
    return this.pager?.loadingMore ?? false;
  }

  set(assets: AssetSummary[], pager: BrowsePager | null = null): void {
    this.assets = assets;
    this.pager = pager;
  }

  clear(): void {
    this.assets = [];
    this.total = undefined;
    this.pager = null;
  }

  requestMore(): Promise<boolean> {
    return this.pager?.loadMore() ?? Promise.resolve(false);
  }

  prefetchNear(id: string): void {
    const idx = this.assets.findIndex((a) => a.id === id);
    if (idx >= 0 && this.assets.length - idx <= PREFETCH_WINDOW) void this.requestMore();
  }

  patch(id: string, fields: Partial<AssetSummary>): void {
    const idx = this.assets.findIndex((a) => a.id === id);
    const current = this.assets[idx];
    if (!current) return;
    this.assets[idx] = { ...current, ...fields };
  }

  insertCopy(copyId: string, source: string, label: string | null): void {
    if (this.assets.some((a) => a.id === copyId)) return;
    const start = this.assets.findIndex((a) => a.id === source);
    const origin = this.assets[start];
    if (!origin) return;
    let at = start + 1;
    while (at < this.assets.length && this.assets[at]?.copyOf === source) at += 1;
    const copy: AssetSummary = {
      ...origin,
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

  positionOf(id: string): number {
    const idx = this.assets.findIndex((a) => a.id === id);
    return this.assets.slice(0, idx + 1).filter((asset) => this.shows(asset)).length;
  }

  prevOf(id: string): AssetSummary | null {
    const idx = this.assets.findIndex((a) => a.id === id);
    if (idx < 0) return null;
    return this.assets.slice(0, idx).findLast((asset) => this.shows(asset)) ?? null;
  }

  nextOf(id: string): AssetSummary | null {
    const idx = this.assets.findIndex((a) => a.id === id);
    if (idx < 0) return null;
    return this.assets.slice(idx + 1).find((asset) => this.shows(asset)) ?? null;
  }

  private shows(asset: AssetSummary): boolean {
    return !browseControls.hides(asset);
  }
}

export const browsing = new BrowsingStore();
