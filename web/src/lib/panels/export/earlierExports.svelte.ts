import type { OnConflict } from '#lib/api/export.ts';
import { findEarlierExports, type EarlierExports as Found } from '#lib/api/exportOutputs.ts';
import { untrack } from 'svelte';
import type { EarlierRun } from './earlierExport';

export interface EarlierQuery {
  assetIds: readonly string[];
  presets: readonly string[];
}

export class EarlierExports {
  policy = $state<OnConflict>('keep_both');

  private found = $state<Found | null>(null);

  private query = $state<EarlierQuery | null>(null);

  private token = 0;

  private key: string | null = null;

  readonly run = $derived.by<EarlierRun | null>(() => {
    if (!this.query || !this.found || this.found.photos === 0) return null;
    return {
      found: this.found,
      total: new Set(this.query.assetIds).size,
      presets: this.query.presets.length
    };
  });

  watch = (query: EarlierQuery | null): void => {
    const key = query ? JSON.stringify([query.assetIds, query.presets]) : null;
    if (key === this.key) return;
    this.key = key;
    untrack(() => {
      this.query = query ? { assetIds: [...query.assetIds], presets: [...query.presets] } : null;
      this.policy = 'keep_both';
      this.found = null;
      void this.load();
    });
  };

  refresh = (): void => {
    void this.load();
  };

  take = (): OnConflict => {
    const policy = this.run ? this.policy : 'keep_both';
    this.policy = 'keep_both';
    return policy;
  };

  private async load(): Promise<void> {
    const token = ++this.token;
    const query = this.query;
    if (!query || query.assetIds.length === 0) return;
    try {
      const found = await findEarlierExports(query.assetIds, query.presets);
      if (token === this.token) this.found = found;
    } catch {
      if (token === this.token) this.found = null;
    }
  }
}
