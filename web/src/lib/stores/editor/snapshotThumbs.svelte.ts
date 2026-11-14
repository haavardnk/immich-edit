import { livePreview } from '#lib/api/preview.ts';
import type { EditSnapshot } from '#lib/api/snapshots.ts';
import type { Edits } from '#lib/types/edits.ts';
import { makeObjectUrl, revoke } from '#lib/utils/objectUrl.ts';

const THUMB_EDGE = 160;

export class SnapshotThumbs {
  urls = $state<Record<string, string>>({});
  private assetId: string | null = null;
  private wanted = new Map<string, Edits>();
  private failed = new Set<string>();
  private controller: AbortController | null = null;
  private current: string | null = null;
  private running = false;

  url(hash: string): string | null {
    return this.urls[hash] ?? null;
  }

  request(assetId: string, snapshots: readonly EditSnapshot[]): void {
    if (this.assetId !== assetId) this.reset();
    this.assetId = assetId;
    this.prune(new Set(snapshots.map((snapshot) => snapshot.manifest_hash)));
    for (const snapshot of snapshots) {
      const hash = snapshot.manifest_hash;
      if (hash in this.urls || this.failed.has(hash) || hash === this.current) continue;
      this.wanted.set(hash, $state.snapshot(snapshot.edits) as Edits);
    }
    void this.drain();
  }

  reset(): void {
    this.controller?.abort();
    for (const url of Object.values(this.urls)) revoke(url);
    this.urls = {};
    this.wanted.clear();
    this.failed.clear();
    this.assetId = null;
  }

  private prune(listed: ReadonlySet<string>): void {
    for (const hash of this.wanted.keys()) if (!listed.has(hash)) this.wanted.delete(hash);
    const entries = Object.entries(this.urls);
    const kept = entries.filter(([hash]) => listed.has(hash));
    if (kept.length === entries.length) return;
    for (const [hash, url] of entries) if (!listed.has(hash)) revoke(url);
    this.urls = Object.fromEntries(kept);
  }

  private async drain(): Promise<void> {
    if (this.running) return;
    this.running = true;
    try {
      for (let next = this.take(); next; next = this.take()) await this.render(...next);
    } finally {
      this.running = false;
      this.controller = null;
      this.current = null;
    }
  }

  private take(): [string, string, Edits] | null {
    const first = this.wanted.entries().next();
    if (first.done || !this.assetId) return null;
    const [hash, edits] = first.value;
    this.wanted.delete(hash);
    return [this.assetId, hash, edits];
  }

  private async render(assetId: string, hash: string, edits: Edits): Promise<void> {
    const controller = new AbortController();
    this.controller = controller;
    this.current = hash;
    try {
      const { blob } = await livePreview(
        assetId,
        edits,
        THUMB_EDGE,
        'none',
        undefined,
        controller.signal,
        'reference'
      );
      if (controller.signal.aborted || this.assetId !== assetId) return;
      this.urls = { ...this.urls, [hash]: makeObjectUrl(blob) };
    } catch {
      if (!controller.signal.aborted && this.assetId === assetId) this.failed.add(hash);
    }
  }
}
