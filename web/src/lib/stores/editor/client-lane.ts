import type { ClientRenderer } from '$lib/render/client-renderer';
import type { RenderView, RenderedFrame, SourceInfo } from '$lib/render/protocol';
import type { Edits } from '$lib/types/edits';
import type { RenderRequest } from '$lib/utils/view-geometry';

export interface ClientJob {
  edits: Edits;
  view: RenderView;
  purpose?: 'color-picker';
  request?: RenderRequest;
}

export interface ClientLaneHooks {
  assetId(): string | null;
  onFrame(job: ClientJob, frame: RenderedFrame, source: SourceInfo, ms: number): void;
  onError(err: unknown): void;
  onIdle(): void;
}

interface SourceWant {
  key: string;
  assetId: string;
  job: ClientJob;
}

export interface SourceSlot {
  stale: boolean;
  key(assetId: string, sensorKey: string, view: RenderView): string;
  load(
    renderer: ClientRenderer,
    assetId: string,
    job: ClientJob,
    signal: AbortSignal
  ): Promise<SourceInfo>;
  view(view: RenderView): RenderView;
}

export const BASE_SLOT: SourceSlot = {
  stale: true,
  key: (assetId, sensorKey) => `${assetId}:${sensorKey}`,
  load: (renderer, assetId, job, signal) =>
    renderer.loadSource(assetId, job.edits, job.view.max_edge, signal),
  view: (view) => view
};

export const TILE_SLOT: SourceSlot = {
  stale: false,
  key: (assetId, sensorKey, view) => `${assetId}:${sensorKey}:${view.roi?.join(',')}`,
  load: (renderer, assetId, job, signal) => {
    const roi = job.view.roi;
    if (!roi) throw new Error('a tile needs a roi');
    return renderer.loadTile(assetId, job.edits, job.view.max_edge, roi, signal);
  },
  view: (view) => ({ ...view, tile: true })
};

export class ClientLane {
  private readonly renderer: ClientRenderer;
  private readonly slot: SourceSlot;
  private readonly hooks: ClientLaneHooks;
  private pending: ClientJob | null = null;
  private last: ClientJob | null = null;
  private running = false;
  private generation = 0;
  private frames = 0;
  private sourceKey: string | null = null;
  private source: SourceInfo | null = null;
  private fetching: string | null = null;
  private wanted: SourceWant | null = null;
  private controller: AbortController | null = null;

  constructor(renderer: ClientRenderer, slot: SourceSlot, hooks: ClientLaneHooks) {
    this.renderer = renderer;
    this.slot = slot;
    this.hooks = hooks;
  }

  get busy(): boolean {
    return this.running || (this.fetching !== null && !this.drawable(this.fetching));
  }

  submit(job: ClientJob): void {
    this.pending = job;
    if (!this.running) void this.pump();
  }

  cancel(): void {
    this.frames++;
    this.pending = null;
    this.last = null;
  }

  reset(): void {
    this.generation++;
    this.cancel();
    this.sourceKey = null;
    this.source = null;
    this.wanted = null;
    this.controller?.abort();
    this.controller = null;
  }

  private async pump(): Promise<void> {
    this.running = true;
    let drawn = false;
    while (this.pending) {
      const job = this.pending;
      this.pending = null;
      this.last = job;
      const frames = this.frames;
      const started = performance.now();
      try {
        const frame = await this.render(job);
        if (!frame) continue;
        if (frames !== this.frames || !this.source) {
          frame.bitmap.close();
          continue;
        }
        drawn = true;
        this.hooks.onFrame(job, frame, this.source, performance.now() - started);
      } catch (err) {
        if (frames === this.frames) this.hooks.onError(err);
      }
    }
    this.running = false;
    if (drawn) this.hooks.onIdle();
  }

  private async render(job: ClientJob): Promise<RenderedFrame | null> {
    const assetId = this.hooks.assetId();
    if (!assetId) return null;
    const inputs = await this.renderer.inputs(job.edits, job.view.max_edge);
    const key = this.slot.key(assetId, inputs.sensor_key, job.view);
    if (key !== this.sourceKey) this.want({ key, assetId, job });
    if (!this.drawable(key)) return null;
    await this.renderer.prepare(inputs);
    return this.renderer.render(job.edits, this.slot.view(job.view));
  }

  private drawable(key: string): boolean {
    return this.sourceKey !== null && (this.slot.stale || key === this.sourceKey);
  }

  private want(next: SourceWant): void {
    if (next.key === this.fetching) {
      this.wanted = null;
      return;
    }
    this.wanted = next;
    if (this.fetching === null) void this.fetchWanted();
  }

  private async fetchWanted(): Promise<void> {
    while (this.wanted) {
      const want = this.wanted;
      this.wanted = null;
      const generation = this.generation;
      const controller = new AbortController();
      this.controller = controller;
      this.fetching = want.key;
      try {
        const source = await this.slot.load(
          this.renderer,
          want.assetId,
          want.job,
          controller.signal
        );
        if (generation !== this.generation) continue;
        this.sourceKey = want.key;
        this.source = source;
        if (this.last && !this.pending) this.submit(this.last);
      } catch (err) {
        if (generation !== this.generation || controller.signal.aborted) continue;
        this.hooks.onError(err);
      } finally {
        this.fetching = null;
        if (this.controller === controller) this.controller = null;
      }
    }
  }
}
