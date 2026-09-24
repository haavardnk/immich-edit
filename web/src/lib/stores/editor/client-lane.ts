import type { ClientRenderer } from '$lib/render/client-renderer';
import type { RenderView, RenderedFrame, SourceInfo } from '$lib/render/protocol';
import type { Edits } from '$lib/types/edits';

export interface ClientJob {
  edits: Edits;
  view: RenderView;
  purpose?: 'color-picker';
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
  edits: Edits;
  maxEdge: number;
}

export class ClientLane {
  private readonly renderer: ClientRenderer;
  private readonly hooks: ClientLaneHooks;
  private pending: ClientJob | null = null;
  private last: ClientJob | null = null;
  private running = false;
  private generation = 0;
  private sourceKey: string | null = null;
  private source: SourceInfo | null = null;
  private fetching: string | null = null;
  private wanted: SourceWant | null = null;
  private controller: AbortController | null = null;

  constructor(renderer: ClientRenderer, hooks: ClientLaneHooks) {
    this.renderer = renderer;
    this.hooks = hooks;
  }

  get busy(): boolean {
    return this.running || (this.sourceKey === null && this.fetching !== null);
  }

  submit(job: ClientJob): void {
    this.pending = job;
    if (!this.running) void this.pump();
  }

  reset(): void {
    this.generation++;
    this.pending = null;
    this.last = null;
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
      const generation = this.generation;
      const started = performance.now();
      try {
        const frame = await this.render(job);
        if (!frame) continue;
        if (generation !== this.generation || !this.source) {
          frame.bitmap.close();
          continue;
        }
        drawn = true;
        this.hooks.onFrame(job, frame, this.source, performance.now() - started);
      } catch (err) {
        if (generation === this.generation) this.hooks.onError(err);
      }
    }
    this.running = false;
    if (drawn) this.hooks.onIdle();
  }

  private async render(job: ClientJob): Promise<RenderedFrame | null> {
    const assetId = this.hooks.assetId();
    if (!assetId) return null;
    const maxEdge = job.view.max_edge;
    const inputs = await this.renderer.inputs(job.edits, maxEdge);
    const key = `${assetId}:${inputs.sensor_key}`;
    if (key !== this.sourceKey) this.want({ key, assetId, edits: job.edits, maxEdge });
    if (this.sourceKey === null) return null;
    await this.renderer.prepare(inputs);
    return this.renderer.render(job.edits, job.view);
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
        const source = await this.renderer.loadSource(
          want.assetId,
          want.edits,
          want.maxEdge,
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
