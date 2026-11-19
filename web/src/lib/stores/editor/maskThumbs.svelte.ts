import { livePreview, maskWeightPreview } from '#lib/api/preview.ts';
import { weightEdits } from '#lib/stores/editor/maskWeight.svelte.ts';
import type { Edits } from '#lib/types/edits.ts';

const THUMB_EDGE = 96;
const SETTLE_MS = 400;

type Want = { assetId: string; edits: Edits };

export class MaskThumbs {
  urls = $state<Record<string, string>>({});
  #keys: Record<string, string> = {};
  #assetId: string | null = null;
  #want: Want | null = null;
  #timer: ReturnType<typeof setTimeout> | null = null;
  #running = false;

  sync(assetId: string | null, edits: Edits, ready: boolean): void {
    if (assetId !== this.#assetId) this.#reset(assetId);
    if (!assetId || !ready) return;
    this.#want = { assetId, edits };
    if (this.#timer) clearTimeout(this.#timer);
    this.#timer = setTimeout(() => {
      this.#timer = null;
      void this.#drain();
    }, SETTLE_MS);
  }

  async #drain(): Promise<void> {
    if (this.#running) return;
    this.#running = true;
    try {
      while (this.#want) {
        const want = this.#want;
        this.#want = null;
        await this.#render(want);
      }
    } finally {
      this.#running = false;
    }
  }

  async #render({ assetId, edits }: Want): Promise<void> {
    const live = new Set(edits.masks.map((layer) => layer.id));
    this.#prune(live);
    for (const layer of edits.masks) {
      if (this.#want || assetId !== this.#assetId) return;
      const solo = weightEdits(edits, layer.id);
      if (!solo) continue;
      const key = JSON.stringify(solo);
      if (this.#keys[layer.id] === key) continue;
      const blob = await livePreview(
        assetId,
        solo,
        THUMB_EDGE,
        maskWeightPreview(layer.id),
        undefined,
        undefined,
        'mask_thumb'
      ).then(
        (res) => res.blob,
        () => null
      );
      if (!blob || assetId !== this.#assetId) continue;
      this.#keys[layer.id] = key;
      this.#set(layer.id, URL.createObjectURL(blob));
    }
  }

  #set(layerId: string, url: string): void {
    const old = this.urls[layerId];
    this.urls = { ...this.urls, [layerId]: url };
    if (old) URL.revokeObjectURL(old);
  }

  #prune(live: Set<string>): void {
    const stale = Object.keys(this.urls).filter((id) => !live.has(id));
    if (stale.length === 0) return;
    const next = { ...this.urls };
    for (const id of stale) {
      const url = next[id];
      if (url) URL.revokeObjectURL(url);
      delete next[id];
      delete this.#keys[id];
    }
    this.urls = next;
  }

  #reset(assetId: string | null): void {
    for (const url of Object.values(this.urls)) URL.revokeObjectURL(url);
    this.urls = {};
    this.#keys = {};
    this.#want = null;
    this.#assetId = assetId;
  }
}

export const maskThumbs = new MaskThumbs();
