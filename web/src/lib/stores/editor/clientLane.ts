import type { ClientRenderer } from '#lib/render/clientRenderer.ts';
import type { RenderInputs, RenderView, RenderedFrame, SourceInfo } from '#lib/render/protocol.ts';
import type { Edits } from '#lib/types/edits.ts';
import type { RenderRequest } from '#lib/utils/viewGeometry.ts';

export interface ClientJob {
  edits: Edits;
  view: RenderView;
  purpose?: 'color-picker';
  request?: RenderRequest;
}

export interface ClientLaneHooks {
  assetId(): string | null;
  sourceRevision(): string;
  onFrame(job: ClientJob, frame: RenderedFrame, source: SourceInfo, ms: number): void;
  onError(err: unknown): void;
  onIdle(): void;
}

interface SourceWant {
  key: string;
  window: string;
  assetId: string;
  job: ClientJob;
}

export interface SourceSlot {
  stale: boolean;
  key(assetId: string, source: string, view: RenderView, inputs: RenderInputs): string;
  context(inputs: RenderInputs): string | null;
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
  key: (assetId, source) => `${assetId}:${source}`,
  context: () => null,
  load: (renderer, assetId, job, signal) =>
    renderer.loadSource(assetId, job.edits, job.view.max_edge, signal),
  view: (view) => view
};

export const TILE_SLOT: SourceSlot = {
  stale: false,
  key: (assetId, source, view, inputs) =>
    `${assetId}:${source}:${inputs.window_key}:${view.roi?.join(',')}`,
  context: (inputs) => inputs.context_key,
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
  private loadedKey: string | null = null;
  private source: SourceInfo | null = null;
  private fetching: SourceWant | null = null;
  private wanted: SourceWant | null = null;
  private controller: AbortController | null = null;

  constructor(renderer: ClientRenderer, slot: SourceSlot, hooks: ClientLaneHooks) {
    this.renderer = renderer;
    this.slot = slot;
    this.hooks = hooks;
  }

  get busy(): boolean {
    return this.running || (this.fetching !== null && !this.drawable(this.fetching.window));
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
    this.loadedKey = null;
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
      try {
        const timed = await this.render(job);
        if (!timed) continue;
        const { frame, ms } = timed;
        if (frames !== this.frames || !this.source) {
          frame.bitmap.close();
          continue;
        }
        drawn = true;
        this.hooks.onFrame(job, frame, this.source, ms);
      } catch (err) {
        if (frames === this.frames) this.hooks.onError(err);
      }
    }
    this.running = false;
    if (drawn) this.hooks.onIdle();
  }

  private async render(job: ClientJob): Promise<{ frame: RenderedFrame; ms: number } | null> {
    const assetId = this.hooks.assetId();
    if (!assetId) return null;
    const inputs = await this.renderer.inputs(job.edits, job.view.max_edge);
    const window = this.slot.key(
      assetId,
      `${inputs.sensor_key}:${this.hooks.sourceRevision()}`,
      job.view,
      inputs
    );
    const key = `${window}|${this.slot.context(inputs)}`;
    const stale = window !== this.sourceKey || (this.source?.context && key !== this.loadedKey);
    if (stale) this.want({ key, window, assetId, job });
    else this.settle();
    if (!this.drawable(window)) return null;
    await this.renderer.prepare(inputs);
    const started = performance.now();
    const frame = await this.renderer.render(job.edits, this.slot.view(job.view));
    return { frame, ms: performance.now() - started };
  }

  private drawable(key: string): boolean {
    return this.sourceKey !== null && (this.slot.stale || key === this.sourceKey);
  }

  private settle(): void {
    this.wanted = null;
    if (this.fetching) this.controller?.abort();
  }

  private want(next: SourceWant): void {
    if (next.key === this.fetching?.key) {
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
      this.fetching = want;
      try {
        const source = await this.slot.load(
          this.renderer,
          want.assetId,
          want.job,
          controller.signal
        );
        if (generation !== this.generation) continue;
        this.sourceKey = want.window;
        this.loadedKey = want.key;
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
