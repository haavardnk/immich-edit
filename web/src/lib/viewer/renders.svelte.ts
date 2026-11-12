import { ApiError } from '#lib/api/client.ts';
import { getEdits } from '#lib/api/edits.ts';
import { livePreview } from '#lib/api/preview.ts';
import { manifestToEdits } from '#lib/edits/manifest.ts';
import type { Edits } from '#lib/types/edits.ts';
import { makeObjectUrl, revoke } from '#lib/utils/objectUrl.ts';
import { isFullFrame } from '#lib/utils/viewGeometry.ts';
import { landedFullEdge, type LandedJob, type ViewerJob } from '#lib/viewer/jobs.ts';

type Running = { job: ViewerJob; abort: AbortController };

function aborted(err: unknown): boolean {
  if (err instanceof DOMException && err.name === 'AbortError') return true;
  return err instanceof ApiError && err.code === 'superseded';
}

export class ViewerRenders {
  base = $state.raw<LandedJob | null>(null);
  left = $state.raw<LandedJob | null>(null);
  tile = $state.raw<LandedJob | null>(null);
  private jobs: readonly ViewerJob[] = [];
  private running: Running | null = null;
  private failed = new Set<string>();
  private saved: { id: string; edits: Edits } | null = null;

  plan(jobs: readonly ViewerJob[]): void {
    this.jobs = jobs;
    this.failed = new Set([...this.failed].filter((key) => jobs.some((job) => job.key === key)));
    this.pump();
  }

  dispose(): void {
    this.jobs = [];
    this.running?.abort.abort();
    for (const landed of [this.base, this.left, this.tile]) revoke(landed?.url ?? null);
    this.base = null;
    this.left = null;
    this.tile = null;
  }

  private next(): ViewerJob | null {
    return (
      this.jobs.find((job) => job.key !== this[job.slot]?.key && !this.failed.has(job.key)) ?? null
    );
  }

  private pump(): void {
    const next = this.next();
    const running = this.running;
    if (!running) {
      if (next) void this.run(next);
      return;
    }
    if (next?.key === running.job.key) return;
    const waits =
      running.job.patient &&
      next?.slot === running.job.slot &&
      next.assetId === running.job.assetId;
    if (!waits) running.abort.abort();
  }

  private async run(job: ViewerJob): Promise<void> {
    const abort = new AbortController();
    this.running = { job, abort };
    try {
      this.land(await this.fetch(job, abort.signal));
    } catch (err) {
      if (!aborted(err)) this.failed.add(job.key);
    } finally {
      this.running = null;
      this.pump();
    }
  }

  private async fetch(job: ViewerJob, signal: AbortSignal): Promise<LandedJob> {
    const edits = job.edits ?? (await this.savedEdits(job.assetId));
    signal.throwIfAborted();
    const { blob } = await livePreview(
      job.assetId,
      edits,
      job.edge,
      'none',
      job.proof,
      signal,
      'viewer',
      job.roi && !isFullFrame(job.roi) ? job.roi : undefined
    );
    const url = makeObjectUrl(blob);
    const image = new Image();
    image.src = url;
    try {
      await image.decode();
      signal.throwIfAborted();
    } catch (err) {
      revoke(url);
      throw err;
    }
    const w = image.naturalWidth;
    const h = image.naturalHeight;
    return { ...job, url, w, h, fullEdge: landedFullEdge(job, w, h) };
  }

  private land(landed: LandedJob): void {
    const previous = this[landed.slot];
    this[landed.slot] = landed;
    revoke(previous?.url ?? null);
  }

  private async savedEdits(assetId: string): Promise<Edits> {
    if (this.saved?.id === assetId) return this.saved.edits;
    const edits = manifestToEdits((await getEdits(assetId)).manifest);
    this.saved = { id: assetId, edits };
    return edits;
  }
}
